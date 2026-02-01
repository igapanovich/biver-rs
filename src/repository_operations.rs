use crate::biver_result::BiverResult;
use crate::env::Env;
use crate::extensions::CountIsAtLeast;
use crate::repository_data::{ContentBlobKind, Head, RepositoryData, Version};
use crate::repository_paths::RepositoryPaths;
use crate::version_id::VersionId;
use crate::{biver_result, hash, image_magick, known_file_types, nickname, xdelta3};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, SystemTime};

const DEFAULT_BRANCH: &str = "main";

const MAX_CONSECUTIVE_PATCHES: usize = 7;

pub fn paths(versioned_file_path: PathBuf) -> RepositoryPaths {
    let extension = match versioned_file_path.extension() {
        Some(extension) => {
            let mut extension = OsString::from(extension);
            extension.push(".biver");
            extension
        }
        None => OsString::from("biver"),
    };

    let repository_dir_path = versioned_file_path.with_extension(extension);

    let data_file_path = repository_dir_path.join("data.json");

    RepositoryPaths {
        versioned_file: versioned_file_path,
        repository_dir: repository_dir_path,
        data_file: data_file_path,
    }
}

pub enum RepositoryDataResult {
    Initialized(RepositoryData),
    NotInitialized,
}

pub fn data(repository_paths: &RepositoryPaths) -> BiverResult<RepositoryDataResult> {
    if !repository_paths.data_file.exists() {
        return Ok(RepositoryDataResult::NotInitialized);
    }

    let data_file_contents = fs::read(&repository_paths.data_file)?;
    let repository_data = serde_json::from_slice(&data_file_contents)?;

    Ok(RepositoryDataResult::Initialized(repository_data))
}

pub enum CommitResult {
    Ok,
    NothingToCommit,
    BranchRequired,
    BranchAlreadyExists,
}

pub fn commit_initial_version(env: &Env, repo_paths: &RepositoryPaths, branch: Option<&str>, description: Option<&str>) -> BiverResult<CommitResult> {
    if !fs::exists(&repo_paths.repository_dir)? {
        fs::create_dir(&repo_paths.repository_dir)?;
    } else if fs::exists(&repo_paths.data_file)? {
        return biver_result::error("The data file already exists.");
    }

    let versioned_file = File::open(&repo_paths.versioned_file)?;
    let versioned_file_xxh3_128 = hash::xxh3_128(&versioned_file)?;
    let versioned_file_length = fs::metadata(&repo_paths.versioned_file)?.len();

    let new_version_id = VersionId::new();

    let branch = branch.unwrap_or(DEFAULT_BRANCH);

    let preview_blob_file_name = if can_create_preview(env, repo_paths) {
        Some(preview_blob_file_name(new_version_id))
    } else {
        None
    };

    let new_version = Version {
        id: new_version_id,
        creation_time: Utc::now(),
        nickname: nickname::new_nickname(versioned_file_xxh3_128),
        versioned_file_length,
        versioned_file_xxh3_128,
        description: description.unwrap_or_default().to_string(),
        parent: None,
        content_blob_file_name: content_blob_file_name(new_version_id),
        content_blob_kind: ContentBlobKind::Full,
        preview_blob_file_name: preview_blob_file_name.clone(),
    };

    let repo_data = RepositoryData {
        head: Head::Branch(branch.to_string()),
        branches: HashMap::from([(branch.to_string(), new_version_id)]),
        versions: vec![new_version.clone()],
    };

    if let Some(preview_blob_file_name) = preview_blob_file_name {
        write_versioned_file_to_preview_blob(env, repo_paths, &preview_blob_file_name)?;
    }
    write_versioned_file_to_content_blob(env, repo_paths, &repo_data, &new_version)?;
    write_data_file(&repo_data, repo_paths)?;

    Ok(CommitResult::Ok)
}

pub fn commit_version(env: &Env, repo_paths: &RepositoryPaths, repo_data: &mut RepositoryData, new_branch: Option<&str>, description: Option<&str>) -> BiverResult<CommitResult> {
    let versioned_file = File::open(&repo_paths.versioned_file)?;
    let versioned_file_xxh3_128 = hash::xxh3_128(&versioned_file)?;
    let versioned_file_length = fs::metadata(&repo_paths.versioned_file)?.len();

    let parent = repo_data.head_version();

    if versioned_file_xxh3_128 == parent.versioned_file_xxh3_128 {
        return Ok(CommitResult::NothingToCommit);
    }

    if let Some(new_branch) = new_branch
        && repo_data.branches.contains_key(new_branch)
    {
        return Ok(CommitResult::BranchAlreadyExists);
    }

    let branch = match (new_branch, repo_data.head.branch()) {
        (Some(new_branch), _) => new_branch.to_string(),
        (None, Some(branch)) => branch.to_string(),
        (None, None) => return Ok(CommitResult::BranchRequired),
    };

    let new_version_id = VersionId::new();

    let content_blob_kind = content_blob_kind_for_child_of(repo_data, parent.id);

    let preview_blob_file_name = if can_create_preview(env, repo_paths) {
        Some(preview_blob_file_name(new_version_id))
    } else {
        None
    };

    let new_version = Version {
        id: new_version_id,
        creation_time: Utc::now(),
        nickname: nickname::new_nickname(versioned_file_xxh3_128),
        versioned_file_length,
        versioned_file_xxh3_128,
        description: description.unwrap_or_default().to_string(),
        parent: Some(parent.id),
        content_blob_file_name: content_blob_file_name(new_version_id),
        content_blob_kind,
        preview_blob_file_name: preview_blob_file_name.clone(),
    };

    repo_data.head = Head::Branch(branch.to_string());
    repo_data.versions.push(new_version.clone());
    repo_data.branches.insert(branch.to_string(), new_version_id);

    if let Some(preview_blob_file_name) = preview_blob_file_name {
        write_versioned_file_to_preview_blob(env, repo_paths, &preview_blob_file_name)?;
    }
    write_versioned_file_to_content_blob(env, repo_paths, &repo_data, &new_version)?;
    write_data_file(repo_data, repo_paths)?;

    Ok(CommitResult::Ok)
}

pub enum AmendResult {
    Ok,
    NoUncommittedChanges,
    HeadMustBeBranch,
    CannotAmendParent,
    HeadEqualsParent,
}

pub fn amend_head(env: &Env, repo_paths: &RepositoryPaths, repo_data: &mut RepositoryData, description: Option<&str>) -> BiverResult<AmendResult> {
    let versioned_file = File::open(&repo_paths.versioned_file)?;
    let versioned_file_xxh3_128 = hash::xxh3_128(&versioned_file)?;
    let versioned_file_length = fs::metadata(&repo_paths.versioned_file)?.len();

    let head = repo_data.head_version();
    let head_id = head.id;

    if versioned_file_xxh3_128 == head.versioned_file_xxh3_128 {
        return Ok(AmendResult::NoUncommittedChanges);
    }

    let Some(head_branch) = repo_data.head.branch() else {
        return Ok(AmendResult::HeadMustBeBranch);
    };

    if repo_data.iter_children(head.id).next().is_some() {
        return Ok(AmendResult::CannotAmendParent);
    }

    if let Some(parent_id) = head.parent
        && repo_data.version(parent_id).unwrap().versioned_file_xxh3_128 == versioned_file_xxh3_128
    {
        return Ok(AmendResult::HeadEqualsParent);
    }

    let new_version_id = VersionId::new();

    let content_blob_kind = match head.parent {
        Some(parent_id) => content_blob_kind_for_child_of(repo_data, parent_id),
        None => ContentBlobKind::Full,
    };

    let preview_blob_file_name = if can_create_preview(env, repo_paths) {
        Some(preview_blob_file_name(new_version_id))
    } else {
        None
    };

    let new_head = Version {
        id: new_version_id,
        creation_time: Utc::now(),
        nickname: nickname::new_nickname(versioned_file_xxh3_128),
        versioned_file_length,
        versioned_file_xxh3_128,
        description: description.unwrap_or(&head.description).to_string(),
        parent: head.parent,
        content_blob_file_name: content_blob_file_name(new_version_id),
        content_blob_kind,
        preview_blob_file_name: preview_blob_file_name.clone(),
    };

    repo_data.branches.insert(head_branch.to_string(), new_version_id);
    repo_data.versions.retain(|v| v.id != head_id);
    repo_data.versions.push(new_head.clone());

    if let Some(preview_blob_file_name) = preview_blob_file_name {
        write_versioned_file_to_preview_blob(env, repo_paths, &preview_blob_file_name)?;
    }
    write_versioned_file_to_content_blob(env, repo_paths, &repo_data, &new_head)?;
    write_data_file(&repo_data, repo_paths)?;

    Ok(AmendResult::Ok)
}

pub enum RewordResult {
    Ok,
    InvalidTarget,
}

pub fn reword(repo_paths: &RepositoryPaths, repo_data: &mut RepositoryData, target: &str, description: &str) -> BiverResult<RewordResult> {
    let Some(target_version) = resolve_target_strict_mut(repo_data, target) else {
        return Ok(RewordResult::InvalidTarget);
    };

    target_version.description = description.to_string();

    write_data_file(repo_data, repo_paths)?;

    Ok(RewordResult::Ok)
}

pub fn has_uncommitted_changes(repo_paths: &RepositoryPaths, repo_data: &RepositoryData) -> BiverResult<bool> {
    let versioned_file_metadata = fs::metadata(&repo_paths.versioned_file)?;
    let head_version = repo_data.head_version();

    if versioned_file_metadata.len() != head_version.versioned_file_length {
        return Ok(true);
    }

    let versioned_file = File::open(&repo_paths.versioned_file)?;

    let current_xxh3_128 = hash::xxh3_128(&versioned_file)?;

    Ok(head_version.versioned_file_xxh3_128 != current_xxh3_128)
}

pub fn discard(env: &Env, repo_paths: &RepositoryPaths, repo_data: &RepositoryData) -> BiverResult<()> {
    let head_version = repo_data.head_version();
    set_versioned_file_to_version(env, repo_paths, repo_data, &head_version)?;
    Ok(())
}

pub enum ResetResult {
    Ok,
    HeadMustBeBranch,
    InvalidTarget,
    CannotLeaveOrphans,
}

pub fn reset(repo_paths: &RepositoryPaths, repo_data: &mut RepositoryData, target: &str) -> BiverResult<ResetResult> {
    let Some(branch) = repo_data.head.branch() else {
        return Ok(ResetResult::HeadMustBeBranch);
    };

    let Some(target_version) = resolve_target_strict(repo_data, target) else {
        return Ok(ResetResult::InvalidTarget);
    };
    let target_version_id = target_version.id;

    let erased_versions: Vec<_> = repo_data.iter_head_and_ancestors().take_while(|v| v.id != target_version.id).collect();

    let erased_versions_have_root = erased_versions.iter().any(|v| v.is_root());
    if erased_versions_have_root {
        return Ok(ResetResult::InvalidTarget);
    }

    let head_has_children = repo_data.iter_children(repo_data.head_version().id).count_is_at_least(1);
    if head_has_children {
        return Ok(ResetResult::CannotLeaveOrphans);
    }

    let erased_versions_have_multi_parents = erased_versions.iter().any(|v| repo_data.iter_children(v.id).count_is_at_least(2));
    if erased_versions_have_multi_parents {
        return Ok(ResetResult::CannotLeaveOrphans);
    }

    let erased_version_ids: Vec<_> = erased_versions.iter().map(|v| v.id).collect();

    repo_data.versions.retain(|v| !erased_version_ids.contains(&v.id));
    repo_data.branches.insert(branch.to_string(), target_version_id);

    write_data_file(&repo_data, repo_paths)?;

    Ok(ResetResult::Ok)
}

pub enum CheckOutResult {
    Ok,
    BlockedByUncommittedChanges,
    InvalidTarget,
}

pub fn check_out(env: &Env, repo_paths: &RepositoryPaths, repo_data: &mut RepositoryData, target: &str) -> BiverResult<CheckOutResult> {
    let has_uncommitted_changes = has_uncommitted_changes(repo_paths, repo_data)?;

    if has_uncommitted_changes {
        return Ok(CheckOutResult::BlockedByUncommittedChanges);
    }

    let new_head = match resolve_target(repo_data, target) {
        TargetResult::Invalid => return Ok(CheckOutResult::InvalidTarget),
        TargetResult::Branch(branch) => Head::Branch(branch.to_string()),
        TargetResult::Version(version) => Head::Version(version.id),
    };

    repo_data.head = new_head;
    let new_head_version = repo_data.head_version();

    write_data_file(repo_data, repo_paths)?;
    set_versioned_file_to_version(env, repo_paths, repo_data, new_head_version)?;

    Ok(CheckOutResult::Ok)
}

pub enum RestoreResult {
    Ok,
    BlockedByUncommittedChanges,
    InvalidTarget,
}

pub fn restore(env: &Env, repo_paths: &RepositoryPaths, repo_data: &RepositoryData, target: &str, output: Option<&Path>) -> BiverResult<RestoreResult> {
    let has_uncommitted_changes = has_uncommitted_changes(repo_paths, repo_data)?;

    if has_uncommitted_changes {
        return Ok(RestoreResult::BlockedByUncommittedChanges);
    }

    let target_version = match resolve_target(repo_data, target) {
        TargetResult::Invalid => return Ok(RestoreResult::InvalidTarget),
        TargetResult::Branch(branch) => repo_data.version(repo_data.branches[branch]).expect("Branch resolved from target must exist"),
        TargetResult::Version(version) => version,
    };

    let output = output.unwrap_or_else(|| &repo_paths.versioned_file);

    write_version_content(env, repo_paths, repo_data, target_version, output)?;

    Ok(RestoreResult::Ok)
}

pub enum VersionResult<'a> {
    Ok(&'a Version),
    InvalidTarget,
}

pub fn version<'a>(repo_data: &'a RepositoryData, target: &str) -> VersionResult<'a> {
    let version = match resolve_target(repo_data, target) {
        TargetResult::Invalid => return VersionResult::InvalidTarget,
        TargetResult::Version(version) => version,
        TargetResult::Branch(branch) => repo_data.branch_leaf(branch).expect("Branch resolved from target must exist"),
    };

    VersionResult::Ok(version)
}

pub enum PreviewResult {
    Ok(PathBuf),
    NoPreviewAvailable,
}

pub fn preview(repo_paths: &RepositoryPaths, version: &Version) -> PreviewResult {
    match version.preview_blob_file_name.as_ref() {
        None => PreviewResult::NoPreviewAvailable,
        Some(preview_file_name) => PreviewResult::Ok(repo_paths.file_path(preview_file_name)),
    }
}

pub enum RenameBranchResult {
    Ok,
    AnotherBranchExistsWithSameName,
    BranchDoesNotExist,
}

pub fn rename_branch(repo_paths: &RepositoryPaths, repo_data: &mut RepositoryData, old_name: &str, new_name: &str) -> BiverResult<RenameBranchResult> {
    if old_name == new_name {
        return Ok(RenameBranchResult::Ok);
    }

    if repo_data.branches.contains_key(new_name) {
        return Ok(RenameBranchResult::AnotherBranchExistsWithSameName);
    }

    let Some(branch_version_id) = repo_data.branches.remove(old_name) else {
        return Ok(RenameBranchResult::BranchDoesNotExist);
    };

    repo_data.branches.insert(new_name.to_string(), branch_version_id);

    write_data_file(repo_data, repo_paths)?;

    Ok(RenameBranchResult::Ok)
}

pub enum DeleteBranchResult {
    Ok,
    BranchDoesNotExist,
    CannotDeleteHead,
}

pub fn delete_branch(repo_paths: &RepositoryPaths, repo_data: &mut RepositoryData, name: &String) -> BiverResult<DeleteBranchResult> {
    if !repo_data.branches.contains_key(name) {
        return Ok(DeleteBranchResult::BranchDoesNotExist);
    }

    let branch_leaf_version_id = repo_data.branches[name];

    let versions_on_other_branches = {
        let mut result = HashSet::new();
        let leaf_ids = repo_data.branches.iter().filter(|(b, _)| *b != name).map(|(_, v)| *v);
        for leaf_id in leaf_ids {
            for version in repo_data.iter_version_and_ancestors(leaf_id) {
                if !result.insert(version.id) {
                    break;
                }
            }
        }
        result
    };

    let erased_version_ids = repo_data
        .iter_version_and_ancestors(branch_leaf_version_id)
        .map(|v| v.id)
        .take_while(|id| !versions_on_other_branches.contains(id))
        .collect::<Vec<_>>();

    let head_version = repo_data.head_version();

    if erased_version_ids.contains(&head_version.id) {
        return Ok(DeleteBranchResult::CannotDeleteHead);
    }

    repo_data.branches.remove(name);
    repo_data.versions.retain(|v| !erased_version_ids.contains(&v.id));

    write_data_file(repo_data, repo_paths)?;

    Ok(DeleteBranchResult::Ok)
}

fn write_data_file(data: &RepositoryData, paths: &RepositoryPaths) -> BiverResult<()> {
    if !data.valid() {
        panic!("Repository data is not valid: {:#?}", data);
    }

    let backup1 = paths.file_path("data_backup1.json");
    let backup2 = paths.file_path("data_backup2.json");
    let backup3 = paths.file_path("data_backup3.json");
    let backup4 = paths.file_path("data_backup4.json");
    let backup5 = paths.file_path("data_backup5.json");

    rotate_backup(&backup4, &backup5, Duration::from_hours(24))?;
    rotate_backup(&backup3, &backup4, Duration::from_hours(5))?;
    rotate_backup(&backup2, &backup3, Duration::from_hours(1))?;
    rotate_backup(&backup1, &backup2, Duration::from_mins(5))?;
    rotate_backup(&paths.data_file, &backup1, Duration::from_secs(10))?;

    let data_file_content = serde_json::to_string_pretty(data)?;
    fs::write(&paths.data_file, data_file_content)?;

    Ok(())
}

fn rotate_backup(previous: &Path, next: &Path, interval: Duration) -> BiverResult<()> {
    if !previous.exists() {
        return Ok(());
    }

    if next.exists() && next.metadata()?.modified()? > SystemTime::now() - interval {
        return Ok(());
    }

    fs::copy(previous, next)?;

    Ok(())
}

fn set_versioned_file_to_version(env: &Env, paths: &RepositoryPaths, data: &RepositoryData, version: &Version) -> BiverResult<()> {
    write_version_content(env, paths, data, version, &paths.versioned_file)
}

fn write_version_content(env: &Env, paths: &RepositoryPaths, data: &RepositoryData, version: &Version, output: &Path) -> BiverResult<()> {
    let blob_path = paths.file_path(&version.content_blob_file_name);

    match version.content_blob_kind {
        ContentBlobKind::Full => {
            fs::copy(&blob_path, output)?;
        }
        ContentBlobKind::Patch(base_version_id) => {
            let base_version = data.version(base_version_id).expect("Version referenced by patch must exist");
            let base_version_blob_path = paths.file_path(&base_version.content_blob_file_name);
            xdelta3::apply_patch(env, base_version_blob_path.as_path(), blob_path.as_path(), output)?;
        }
    }

    Ok(())
}

enum TargetResult<'b, 'v> {
    Branch(&'b str),
    Version(&'v Version),
    Invalid,
}

fn resolve_target<'b, 'v>(repo_data: &'v RepositoryData, target: &'b str) -> TargetResult<'b, 'v> {
    if target.is_empty() {
        return TargetResult::Invalid;
    }

    // As branch name
    if repo_data.branches.contains_key(target) {
        return TargetResult::Branch(target);
    }

    // As version ID
    let target_as_version_id = VersionId::from_bs58(target);

    if let Some(target_as_version_id) = target_as_version_id {
        let version = repo_data.versions.iter().find(|v| v.id == target_as_version_id);
        if let Some(version) = version {
            return TargetResult::Version(version);
        }
    }

    // As offset
    if target == "~" {
        return TargetResult::Version(repo_data.head_version());
    }

    if target.chars().nth(0) == Some('~')
        && let Ok(offset) = usize::from_str(&target[1..])
    {
        let target_version = repo_data.iter_head_and_ancestors().nth(offset);
        return match target_version {
            None => TargetResult::Invalid,
            Some(target_version) => TargetResult::Version(target_version),
        };
    }

    // As version nickname
    let mut versions: Vec<_> = repo_data.versions.iter().collect();
    versions.sort_by(|a, b| b.creation_time.cmp(&a.creation_time));

    let version = versions.iter().find(|v| nickname_matches(&v.nickname, target));

    if let Some(version) = version {
        return TargetResult::Version(version);
    }

    TargetResult::Invalid
}

fn resolve_target_strict_mut<'v>(repo_data: &'v mut RepositoryData, target: &str) -> Option<&'v mut Version> {
    if target.is_empty() {
        return None;
    }

    let target_as_version_id = VersionId::from_bs58(target);

    if let Some(target_as_version_id) = target_as_version_id {
        let version = repo_data.versions.iter_mut().find(|v| v.id == target_as_version_id);
        if let Some(version) = version {
            return Some(version);
        }
    }

    None
}

fn resolve_target_strict<'v>(repo_data: &'v RepositoryData, target: &str) -> Option<&'v Version> {
    if target.is_empty() {
        return None;
    }

    let target_as_version_id = VersionId::from_bs58(target);

    if let Some(target_as_version_id) = target_as_version_id {
        let version = repo_data.versions.iter().find(|v| v.id == target_as_version_id);
        if let Some(version) = version {
            return Some(version);
        }
    }

    None
}

fn nickname_matches(nickname: &str, input: &str) -> bool {
    if nickname.eq_ignore_ascii_case(input) {
        return true;
    }

    fn nickname_without_dash_matches(nickname: &str, input: &str) -> bool {
        let pairs = nickname.chars().filter(|c| c != &'-').zip(input.chars());

        let mut zip_length = 0;

        for (nickname_char, input_char) in pairs {
            zip_length += 1;

            if !nickname_char.eq_ignore_ascii_case(&input_char) {
                return false;
            }
        }

        zip_length == input.len()
    }

    if nickname_without_dash_matches(nickname, input) {
        return true;
    }

    fn nickname_initials_match(nickname: &str, input: &str) -> bool {
        if input.len() != 2 {
            return false;
        }

        let input_initials_first = input.chars().nth(0).unwrap();
        let input_initials_second = input.chars().nth(1).unwrap();

        let index_of_dash = nickname.find('-').unwrap();
        let nickname_initials_first = nickname.chars().nth(0).unwrap();
        let nickname_initials_second = nickname.chars().nth(index_of_dash + 1).unwrap();

        input_initials_first.eq_ignore_ascii_case(&nickname_initials_first) && input_initials_second.eq_ignore_ascii_case(&nickname_initials_second)
    }

    nickname_initials_match(nickname, input)
}

fn content_blob_file_name(version_id: VersionId) -> String {
    version_id.to_file_name() + "_content"
}

fn preview_blob_file_name(version_id: VersionId) -> String {
    version_id.to_file_name() + "_preview"
}

fn content_blob_kind_for_child_of(repo_data: &RepositoryData, parent_version_id: VersionId) -> ContentBlobKind {
    let patch_sequence_count = repo_data.iter_ancestors(parent_version_id).take_while(|v| v.content_blob_kind.is_patch()).count() + 1;
    if patch_sequence_count >= MAX_CONSECUTIVE_PATCHES {
        ContentBlobKind::Full
    } else {
        ContentBlobKind::Patch(parent_version_id)
    }
}

fn write_versioned_file_to_content_blob(env: &Env, repo_paths: &RepositoryPaths, repository_data: &RepositoryData, version: &Version) -> BiverResult<()> {
    let content_blob_file_path = repo_paths.file_path(&version.content_blob_file_name);

    match version.content_blob_kind {
        ContentBlobKind::Full => {
            fs::copy(&repo_paths.versioned_file, content_blob_file_path)?;
        }
        ContentBlobKind::Patch(base_version_id) => {
            let base_version = repository_data.version(base_version_id).unwrap();
            let base_blob_file_path = repo_paths.file_path(&base_version.content_blob_file_name);
            xdelta3::create_patch(env, base_blob_file_path.as_path(), &repo_paths.versioned_file, content_blob_file_path.as_path())?;
        }
    }

    Ok(())
}

fn can_create_preview(env: &Env, repo_paths: &RepositoryPaths) -> bool {
    if !image_magick::ready(env) {
        return false;
    }

    let Some(versioned_file_extension) = repo_paths.versioned_file.extension().and_then(|e| e.to_str()) else {
        return false;
    };

    known_file_types::is_image(versioned_file_extension)
}

fn write_versioned_file_to_preview_blob(env: &Env, repo_paths: &RepositoryPaths, preview_blob_file_name: &str) -> BiverResult<()> {
    let preview_blob_file_path = repo_paths.file_path(preview_blob_file_name);

    image_magick::create_preview(env, repo_paths.versioned_file.as_path(), preview_blob_file_path.as_path())?;

    Ok(())
}

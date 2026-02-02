use crate::biver_result::BiverResult;
use crate::env::Env;
use crate::repository_data::{ContentBlob, Head, RepositoryData, Version};
use crate::repository_paths::RepositoryPaths;
use crate::version_id::VersionId;
use crate::{biver_result, hash, image_magick, known_file_types, nickname, repository_io};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use crate::extensions::CountIsAtLeast;

const DEFAULT_BRANCH: &str = "main";

const MAX_CONSECUTIVE_PATCHES: usize = 7;

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

    let new_version = Version {
        id: new_version_id,
        creation_time: Utc::now(),
        nickname: nickname::new_nickname(versioned_file_xxh3_128),
        versioned_file_length,
        versioned_file_xxh3_128,
        description: description.unwrap_or_default().to_string(),
        parent: None,
        content_blob: ContentBlob::Full {
            full_blob_file_name: content_blob_file_name(new_version_id),
        },
        preview_blob_file_name: preview_blob_file_name(env, repo_paths, new_version_id),
    };

    let repo_data = RepositoryData {
        head: Head::Branch(branch.to_string()),
        branches: HashMap::from([(branch.to_string(), new_version_id)]),
        versions: vec![new_version.clone()],
    };

    if let Some(preview_blob_file_name) = new_version.preview_blob_file_name {
        repository_io::store_version_preview(env, repo_paths, &preview_blob_file_name, &repo_paths.versioned_file)?;
    }
    repository_io::store_version_content(env, repo_paths, &new_version.content_blob, &repo_paths.versioned_file)?;
    repository_io::write_data(repo_paths, &repo_data)?;

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

    let content_blob = content_blob(repo_data, Some(parent.id), content_blob_file_name(new_version_id));

    let new_version = Version {
        id: new_version_id,
        creation_time: Utc::now(),
        nickname: nickname::new_nickname(versioned_file_xxh3_128),
        versioned_file_length,
        versioned_file_xxh3_128,
        description: description.unwrap_or_default().to_string(),
        parent: Some(parent.id),
        content_blob,
        preview_blob_file_name: preview_blob_file_name(env, repo_paths, new_version_id),
    };

    repo_data.head = Head::Branch(branch.to_string());
    repo_data.versions.push(new_version.clone());
    repo_data.branches.insert(branch.to_string(), new_version_id);

    if let Some(preview_blob_file_name) = new_version.preview_blob_file_name {
        repository_io::store_version_preview(env, repo_paths, &preview_blob_file_name, &repo_paths.versioned_file)?;
    }
    repository_io::store_version_content(env, repo_paths, &new_version.content_blob, &repo_paths.versioned_file)?;
    repository_io::write_data(repo_paths, repo_data)?;

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

    let new_head = Version {
        id: new_version_id,
        creation_time: Utc::now(),
        nickname: nickname::new_nickname(versioned_file_xxh3_128),
        versioned_file_length,
        versioned_file_xxh3_128,
        description: description.unwrap_or(&head.description).to_string(),
        parent: head.parent,
        content_blob: content_blob(repo_data, head.parent, content_blob_file_name(new_version_id)),
        preview_blob_file_name: preview_blob_file_name(env, repo_paths, new_version_id),
    };

    repo_data.branches.insert(head_branch.to_string(), new_version_id);
    repo_data.versions.retain(|v| v.id != head_id);
    repo_data.versions.push(new_head);

    let new_head = repo_data.head_version();

    if let Some(preview_blob_file_name) = &new_head.preview_blob_file_name {
        repository_io::store_version_preview(env, repo_paths, &preview_blob_file_name, &repo_paths.versioned_file)?;
    }
    repository_io::store_version_content(env, repo_paths, &new_head.content_blob, &repo_paths.versioned_file)?;
    repository_io::write_data(repo_paths, &repo_data)?;

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

    repository_io::write_data(repo_paths, repo_data)?;

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
    repository_io::extract_version_content(env, repo_paths, &head_version.content_blob, &repo_paths.versioned_file)?;
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

    let Some(target_version) = resolve_version_id_target(repo_data, target) else {
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

    repository_io::write_data(repo_paths, &repo_data)?;

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

    repository_io::write_data(repo_paths, repo_data)?;
    repository_io::extract_version_content(env, repo_paths, &new_head_version.content_blob, &repo_paths.versioned_file)?;

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

    repository_io::extract_version_content(env, repo_paths, &target_version.content_blob, output)?;

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

    repository_io::write_data(repo_paths, repo_data)?;

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

    repository_io::write_data(repo_paths, repo_data)?;

    Ok(DeleteBranchResult::Ok)
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

fn resolve_version_id_target<'v>(repo_data: &'v RepositoryData, target: &str) -> Option<&'v Version> {
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

fn content_blob(repo_data: &RepositoryData, parent_id: Option<VersionId>, version_content_blob_file_name: String) -> ContentBlob {
    let Some(parent_id) = parent_id else {
        return ContentBlob::Full {
            full_blob_file_name: version_content_blob_file_name,
        };
    };

    let closest_ancestor_full_blob_info = repo_data
        .iter_version_and_ancestors(parent_id)
        .enumerate()
        .filter_map(|(index, ancestor)| {
            if let ContentBlob::Full { full_blob_file_name } = &ancestor.content_blob {
                Some((index, full_blob_file_name))
            } else {
                None
            }
        })
        .next();

    let Some((closest_ancestor_full_blob_index, closest_ancestor_full_blob_file_name)) = closest_ancestor_full_blob_info else {
        return ContentBlob::Full {
            full_blob_file_name: version_content_blob_file_name,
        };
    };

    if closest_ancestor_full_blob_index >= MAX_CONSECUTIVE_PATCHES {
        ContentBlob::Full {
            full_blob_file_name: version_content_blob_file_name,
        }
    } else {
        ContentBlob::Patch {
            base_blob_file_name: closest_ancestor_full_blob_file_name.clone(),
            patch_blob_file_name: version_content_blob_file_name,
        }
    }
}

fn preview_blob_file_name(env: &Env, repo_paths: &RepositoryPaths, version_id: VersionId) -> Option<String> {
    if can_create_preview(env, repo_paths) {
        let file_name = version_id.to_file_name() + "_preview";
        Some(file_name)
    } else {
        None
    }
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

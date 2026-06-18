# Version

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**version_number** | **String** |  | 
**source_type** | Option<[**models::VersionSourceType**](VersionSourceType.md)> |  | 
**env_vars** | Option<[**Vec<models::EnvVar>**](EnvVar.md)> |  | [optional]
**apply_env_vars_to_build** | Option<**bool**> |  | [optional]
**build_tag** | Option<**String**> |  | [optional]
**source_files** | Option<[**Vec<models::VersionSourceFilesInner>**](VersionSourceFilesInner.md)> |  | [optional]
**git_repo_url** | Option<**String**> | URL of the Git repository when sourceType is GIT_REPO. | [optional]
**tarball_url** | Option<**String**> | URL of the tarball when sourceType is TARBALL. | [optional]
**git_hub_gist_url** | Option<**String**> | URL of the GitHub Gist when sourceType is GITHUB_GIST. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)



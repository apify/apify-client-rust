# UpdateLimitsRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**max_monthly_usage_usd** | Option<**f64**> | If your platform usage in the billing period exceeds the prepaid usage, you will be charged extra. Setting this property you can update your hard limit on monthly platform usage to prevent accidental overage or to limit the extra charges.  | [optional]
**data_retention_days** | Option<**i32**> | Apify securely stores your ten most recent Actor runs indefinitely, ensuring they are always accessible. Unnamed storages and other Actor runs are automatically deleted after the retention period. If you're subscribed, you can change it to keep data for longer or to limit your usage. [Lear more](https://docs.apify.com/platform/storage/usage#data-retention).  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)



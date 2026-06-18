# DatasetFieldStatistics

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**min** | Option<**f64**> | Minimum value of the field. For numbers, this is calculated directly. For strings, this is the length of the shortest string. For arrays, this is the length of the shortest array. For objects, this is the number of keys in the smallest object. | [optional]
**max** | Option<**f64**> | Maximum value of the field. For numbers, this is calculated directly. For strings, this is the length of the longest string. For arrays, this is the length of the longest array. For objects, this is the number of keys in the largest object. | [optional]
**null_count** | Option<**i32**> | How many items in the dataset have a null value for this field. | [optional]
**empty_count** | Option<**i32**> | How many items in the dataset are `undefined`, meaning that for example empty string is not considered empty. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)



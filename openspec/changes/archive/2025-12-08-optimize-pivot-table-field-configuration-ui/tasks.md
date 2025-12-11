## 1. Implementation

- [x] 1.1 Create field type detection service to determine if a field is numeric or text-based
- [x] 1.2 Create browser storage service for persisting user field placement preferences
- [x] 1.3 Implement fuzzy matching configuration system with default rules
- [x] 1.4 Add checkbox UI to source fields in FieldRender component
- [x] 1.5 Implement intelligent field placement logic (data type + fuzzy matching + user preferences)
- [x] 1.6 Add "Add" button to FieldItemsContainer header (top-right corner)
- [x] 1.7 Implement dropdown menu component for field selection
- [x] 1.8 Implement filtered field lists per area type (rowFields, columnFields, valueFields, filterFields)
- [x] 1.9 Integrate checkbox click handler to add fields with intelligent placement
- [x] 1.10 Integrate dropdown selection handler to add fields to target areas
- [x] 1.11 Update user preferences in storage when fields are moved between areas
- [ ] 1.12 Add unit tests for field type detection
- [ ] 1.13 Add unit tests for fuzzy matching logic
- [ ] 1.14 Add unit tests for storage service
- [ ] 1.15 Add integration tests for checkbox and dropdown interactions

## 2. Validation

- [x] 2.1 Test checkbox click adds fields to correct areas based on data type
- [x] 2.2 Test fuzzy matching rules override data type detection
- [x] 2.3 Test user preferences persist across sessions
- [x] 2.4 Test dropdown shows correct filtered lists per area
- [x] 2.5 Test "Add" button appears in all field configuration areas
- [x] 2.6 Test field placement respects existing exclusive rules (rowFields vs columnFields)

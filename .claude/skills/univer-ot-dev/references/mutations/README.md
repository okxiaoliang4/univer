# Mutation Conflict Analysis Documents

This directory contains detailed conflict analysis documents for each mutation in the Univer OT system.

## Purpose

Each document serves as:
- **Complete transform coverage checklist** - Ensures no mutations are forgotten
- **Conflict resolution reference** - Documents how conflicts are resolved
- **Implementation roadmap** - Tracks progress towards 100% coverage
- **Team knowledge base** - Captures OT design decisions and rationale

## Structure

```
mutations/
├── TEMPLATE.md                    # Template for new documents
├── README.md                      # This file
├── insert_row.md                  # InsertRowMutation analysis
├── set_range_values.md            # SetRangeValuesMutation analysis
└── [mutation_name].md             # One file per mutation
```

## Document Format

Each mutation document follows the template structure:

1. **Mutation Overview**
   - Purpose and parameters
   - Affected dimensions (rows, columns, cells, properties)

2. **Conflict Dimensions Analysis**
   - Analysis across all 4 dimensions (unitId, subUnitId, position, feature ID)

3. **Transform Coverage Matrix**
   - Complete list of ALL mutations (53 core + feature plugins)
   - Implementation status (✅ Implemented, 🚧 In Progress, ⏳ Planned, ❌ Not needed)
   - Resolution strategy for each
   - Brief notes

4. **Detailed Conflict Resolution**
   - For each non-identity transform:
     - Conflict analysis
     - Resolution strategy
     - Implementation pseudocode/actual code
     - Test cases
     - Implementation status

5. **Implementation Checklist**
   - Registration status
   - Test coverage
   - Documentation completeness

6. **Last Updated**
   - Date, developer, changes

## Workflow Integration

These documents are **mandatory** and updated throughout the TDD workflow:

### Step 1: Create/Update Mutation
- **Action**: Create new document from TEMPLATE.md
- **Fill**: Mutation overview, parameters, affected dimensions

### Step 2: Create/Update Conflict Analysis Document
- **Action**: Complete conflict dimensions analysis
- **Fill**: Transform coverage matrix (list ALL mutations)

### Step 3: Analyze Conflict Dimensions
- **Action**: For each mutation, determine conflict scenario
- **Update**: Coverage matrix with strategies

### Step 4: Plan Transform Strategy
- **Action**: Choose resolution strategies
- **Update**: Document with chosen strategies and rationale

### Step 5: Write Tests FIRST
- **Action**: Write test cases
- **Update**: Document with test scenarios

### Step 6: Implement Transform
- **Action**: Implement transform functions
- **Update**: Document with code snippets

### Step 7: Run Tests and Iterate
- **Action**: Test and fix
- **Update**: Mark completed tests as ✅

### Step 8: Register in Module
- **Action**: Register transforms
- **Update**: Registration checklist

### Step 9: Coverage Validation
- **Action**: Verify all transforms implemented
- **Update**: Final status verification, Last Updated section

## Benefits

1. **Completeness**: Matrix forces consideration of ALL mutations
2. **Traceability**: Clear mapping from analysis → implementation → tests
3. **Collaboration**: Team members can see progress and pending work
4. **Onboarding**: New developers understand OT system through documents
5. **Maintenance**: Easy to identify gaps when adding new mutations

## Creating a New Document

1. Copy `TEMPLATE.md` to `[mutation_name].md`
2. Fill in mutation overview and parameters
3. Complete transform coverage matrix (list all 53+ mutations)
4. Follow TDD workflow, updating document at each step
5. Keep document in sync with code changes

## Example: InsertRowMutation Document

A complete document includes:
- Overview of what InsertRowMutation does
- Analysis of how it conflicts on each dimension
- Matrix with 53+ rows showing transform status for each mutation
- Detailed sections for key transforms (vs SetRangeValues, vs RemoveRows, etc.)
- Implementation code snippets
- Test case checklists
- Current status and last update information

## Maintenance

**CRITICAL**: These documents must be kept up-to-date:

- When adding a new mutation → Create document + update ALL existing mutation documents to include the new one in their matrices
- When implementing a transform → Update BOTH mutation documents involved (mark as ✅)
- When changing a transform → Update document with new implementation
- When tests pass → Update test checklist status

Stale documents defeat the purpose. Make documentation updates part of your definition of "done".

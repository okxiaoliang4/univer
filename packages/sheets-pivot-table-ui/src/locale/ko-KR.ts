/**
 * Copyright 2023-present DreamNum Co., Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import type enUS from './en-US';

const locale: typeof enUS = {
    pivotTable: {
        menu: {
            insert: '피벗 테이블',
        },
        dialog: {
            createTitle: '피벗 테이블 만들기',
            sourceRangeLabel: '원본 데이터 범위 선택',
            targetRangeLabel: '피벗 테이블 배치 위치 선택',
            targetRangeTypeNew: '새 시트',
            targetRangeTypeExisting: '기존 시트',
            sourceRangeSingleCellError: '원본 범위는 여러 셀을 포함해야 합니다',
            sourceRangeSingleRowError: '원본 범위는 최소 두 행(헤더 및 데이터)을 포함해야 합니다',
            sourceRangeWithMergeError: '원본 범위는 병합된 셀과 겹칠 수 없습니다',
            targetRangeSingleCellError: '대상 위치는 단일 셀이어야 합니다',
            invalidWorksheet: '유효하지 않은 워크시트',
            cancel: '취소',
            confirm: '확인',
        },
        editor: {
            title: '피벗 테이블 필드',
            availableFields: '사용 가능한 필드',
            filters: '필터',
            columns: '열',
            rows: '행',
            values: '값',
            dragFieldsHere: '필드를 여기로 끌어오세요',
            remove: '제거',
            add: '추가',
            aggregationMethodLabel: '집계 방법',
            aggregationType: {
                sum: '합계',
                count: '개수',
                average: '평균',
                min: '최소값',
                max: '최대값',
            },
            containerLabel: '열 {0}',
            containerLabels: {
                sourceFields: '사용 가능한 필드',
                filterFields: '필터',
                columnFields: '열',
                rowFields: '행',
                valueFields: '값',
            },
        },
        panel: {
            sourceRangeLabel: '원본 범위:',
            fieldConfigurationLabel: '필드 구성:',
        },
    },
};

export default locale;

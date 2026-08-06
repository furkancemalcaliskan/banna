import type { FullAuditedEntityDto, PagedAndSortedResultRequestDto } from '@abp/ng.core';

export interface ${entity_name}CreateDto {
${angular_proxy_create_fields}
}

export interface ${entity_name}Dto extends FullAuditedEntityDto<string> {
${angular_proxy_dto_fields}
  concurrencyStamp?: string;
}

export interface ${entity_name}UpdateDto {
${angular_proxy_update_fields}
  concurrencyStamp?: string;
}

export interface ${entity_name}ExcelDownloadDto {
  downloadToken?: string;
  filterText?: string | null;
${angular_proxy_excel_fields}
}

export interface ${entity_name}WithNavigationPropertiesDto {
  ${entity_name_lower}?: ${entity_name}Dto;
${angular_proxy_with_nav_fields}
}

export interface Get${entity_plural}Input extends PagedAndSortedResultRequestDto {
  filterText?: string | null;
${angular_proxy_filter_fields}
}

export interface LookupDto<T> {
  id?: T;
  displayName?: string;
}

export interface LookupRequestDto extends PagedAndSortedResultRequestDto {
  filter?: string | null;
}

export interface DownloadTokenResultDto {
  token?: string;
}

${angular_proxy_nav_models}

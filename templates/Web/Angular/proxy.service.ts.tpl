import { Rest, RestService } from '@abp/ng.core';
import type { PagedResultDto } from '@abp/ng.core';
import { Injectable, inject } from '@angular/core';
import type {
  DownloadTokenResultDto,
  ${entity_name}CreateDto,
  ${entity_name}Dto,
  ${entity_name}ExcelDownloadDto,
  ${entity_name}UpdateDto,
  ${entity_name}WithNavigationPropertiesDto,
  Get${entity_plural}Input,
  LookupDto,
  LookupRequestDto,
} from './models';

@Injectable({ providedIn: 'root' })
export class ${entity_plural}Service {
  private readonly restService = inject(RestService);
  readonly apiName = 'Default';

  create = (input: ${entity_name}CreateDto, config?: Partial<Rest.Config>) =>
    this.restService.request<unknown, ${entity_name}Dto>(
      { method: 'POST', url: '/api/app/${entity_plural_kebab}', body: input },
      { apiName: this.apiName, ...config },
    );

  delete = (id: string, config?: Partial<Rest.Config>) =>
    this.restService.request<unknown, void>(
      { method: 'DELETE', url: `/api/app/${entity_plural_kebab}/${id}` },
      { apiName: this.apiName, ...config },
    );

  deleteAll = (input: Get${entity_plural}Input, config?: Partial<Rest.Config>) =>
    this.restService.request<unknown, void>(
      {
        method: 'DELETE',
        url: '/api/app/${entity_plural_kebab}/all',
        params: { ${angular_proxy_delete_filter_params} },
      },
      { apiName: this.apiName, ...config },
    );

  deleteByIds = (ids: readonly string[], config?: Partial<Rest.Config>) =>
    this.restService.request<unknown, void>(
      { method: 'DELETE', url: '/api/app/${entity_plural_kebab}/by-ids', params: { ids } },
      { apiName: this.apiName, ...config },
    );

  get = (id: string, config?: Partial<Rest.Config>) =>
    this.restService.request<unknown, ${entity_name}Dto>(
      { method: 'GET', url: `/api/app/${entity_plural_kebab}/${id}` },
      { apiName: this.apiName, ...config },
    );

  getDownloadToken = (config?: Partial<Rest.Config>) =>
    this.restService.request<unknown, DownloadTokenResultDto>(
      { method: 'GET', url: '/api/app/${entity_plural_kebab}/download-token' },
      { apiName: this.apiName, ...config },
    );

  getList = (input: Get${entity_plural}Input, config?: Partial<Rest.Config>) =>
    this.restService.request<unknown, PagedResultDto<${entity_name}WithNavigationPropertiesDto>>(
      {
        method: 'GET',
        url: '/api/app/${entity_plural_kebab}',
        params: { ${angular_proxy_filter_params} },
      },
      { apiName: this.apiName, ...config },
    );

  getListAsExcelFile = (
    input: ${entity_name}ExcelDownloadDto,
    config?: Partial<Rest.Config>,
  ) =>
    this.restService.request<unknown, Blob>(
      {
        method: 'GET',
        responseType: 'blob',
        url: '/api/app/${entity_plural_kebab}/as-excel-file',
        params: { ${angular_proxy_excel_params} },
      },
      { apiName: this.apiName, ...config },
    );

  getWithNavigationProperties = (id: string, config?: Partial<Rest.Config>) =>
    this.restService.request<unknown, ${entity_name}WithNavigationPropertiesDto>(
      { method: 'GET', url: `/api/app/${entity_plural_kebab}/${id}/with-navigation-properties` },
      { apiName: this.apiName, ...config },
    );

${angular_proxy_lookup_methods}

  update = (id: string, input: ${entity_name}UpdateDto, config?: Partial<Rest.Config>) =>
    this.restService.request<unknown, ${entity_name}Dto>(
      { method: 'PUT', url: `/api/app/${entity_plural_kebab}/${id}`, body: input },
      { apiName: this.apiName, ...config },
    );
}

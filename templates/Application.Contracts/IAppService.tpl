using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Volo.Abp.Application.Dtos;
using Volo.Abp.Application.Services;
using Volo.Abp.Content;
using ${domain_name}.Shared;

namespace ${namespace}
{
    public interface I${entity_plural}AppService : IApplicationService
    {
        Task<PagedResultDto<${entity_name}WithNavigationPropertiesDto>> GetListAsync(Get${entity_plural}Input input);
        Task<${entity_name}WithNavigationPropertiesDto> GetWithNavigationPropertiesAsync(Guid id);
        Task<${entity_name}Dto> GetAsync(Guid id);
        Task DeleteAsync(Guid id);
        Task<${entity_name}Dto> CreateAsync(${entity_name}CreateDto input);
        Task<${entity_name}Dto> UpdateAsync(Guid id, ${entity_name}UpdateDto input);
        Task<IRemoteStreamContent> GetListAsExcelFileAsync(${entity_name}ExcelDownloadDto input);
        Task DeleteByIdsAsync(List<Guid> ids);
        Task DeleteAllAsync(Get${entity_plural}Input input);
        ${nav_lookup_interface_methods}
        Task<DownloadTokenResultDto> GetDownloadTokenAsync();
    }
}

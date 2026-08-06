using System;
using System.IO;
using System.Linq;
using System.Collections.Generic;
using System.Threading.Tasks;
using System.Linq.Dynamic.Core;
using Microsoft.AspNetCore.Authorization;
using Microsoft.Extensions.Caching.Distributed;
using Volo.Abp;
using Volo.Abp.Authorization;
using Volo.Abp.Caching;
using Volo.Abp.Application.Dtos;
using Volo.Abp.Application.Services;
using Volo.Abp.Content;
using Volo.Abp.Domain.Repositories;
using MiniExcelLibs;
using ${domain_name}.Permissions;
using ${domain_name}.Shared;
${nav_usings}

namespace ${namespace}
{
    [Authorize(${domain_short}Permissions.${entity_plural}.Default)]
    public class ${entity_plural}AppService : ${domain_short}AppService, I${entity_plural}AppService
    {
        protected IDistributedCache<${entity_name}DownloadTokenCacheItem, string> _downloadTokenCache;
        protected I${entity_name}Repository _${entity_name_lower}Repository;
        protected ${entity_name}Manager _${entity_name_lower}Manager;
        ${nav_repo_fields}

        public ${entity_plural}AppService(
            I${entity_name}Repository ${entity_name_lower}Repository,
            ${entity_name}Manager ${entity_name_lower}Manager,
            IDistributedCache<${entity_name}DownloadTokenCacheItem, string> downloadTokenCache${nav_repo_ctor_params}
        )
        {
            _downloadTokenCache = downloadTokenCache;
            _${entity_name_lower}Repository = ${entity_name_lower}Repository;
            _${entity_name_lower}Manager = ${entity_name_lower}Manager;
            ${nav_repo_ctor_assignments}
        }

        public virtual async Task<PagedResultDto<${entity_name}WithNavigationPropertiesDto>> GetListAsync(Get${entity_plural}Input input)
        {
            var totalCount = await _${entity_name_lower}Repository.GetCountAsync(${input_filter_call_args});
            var items = await _${entity_name_lower}Repository.GetListWithNavigationPropertiesAsync(${input_filter_call_args}, input.Sorting, input.MaxResultCount, input.SkipCount);

            return new PagedResultDto<${entity_name}WithNavigationPropertiesDto>
            {
                TotalCount = totalCount,
                Items = ObjectMapper.Map<List<${entity_name}WithNavigationProperties>, List<${entity_name}WithNavigationPropertiesDto>>(items)
            };
        }

        public virtual async Task<${entity_name}WithNavigationPropertiesDto> GetWithNavigationPropertiesAsync(Guid id)
        {
            return ObjectMapper.Map<${entity_name}WithNavigationProperties, ${entity_name}WithNavigationPropertiesDto>(
                await _${entity_name_lower}Repository.GetWithNavigationPropertiesAsync(id));
        }

        public virtual async Task<${entity_name}Dto> GetAsync(Guid id)
        {
            return ObjectMapper.Map<${entity_name}, ${entity_name}Dto>(await _${entity_name_lower}Repository.GetAsync(id));
        }
        ${nav_lookup_methods}

        [Authorize(${domain_short}Permissions.${entity_plural}.Delete)]
        public virtual async Task DeleteAsync(Guid id)
        {
            await _${entity_name_lower}Repository.DeleteAsync(id);
        }

        [Authorize(${domain_short}Permissions.${entity_plural}.Create)]
        public virtual async Task<${entity_name}Dto> CreateAsync(${entity_name}CreateDto input)
        {
            ${required_guid_checks}
            var entity = await _${entity_name_lower}Manager.CreateAsync(${appservice_create_args});
            return ObjectMapper.Map<${entity_name}, ${entity_name}Dto>(entity);
        }

        [Authorize(${domain_short}Permissions.${entity_plural}.Edit)]
        public virtual async Task<${entity_name}Dto> UpdateAsync(Guid id, ${entity_name}UpdateDto input)
        {
            ${required_guid_checks}
            var entity = await _${entity_name_lower}Manager.UpdateAsync(id, ${appservice_update_args});
            return ObjectMapper.Map<${entity_name}, ${entity_name}Dto>(entity);
        }

        [AllowAnonymous]
        public virtual async Task<IRemoteStreamContent> GetListAsExcelFileAsync(${entity_name}ExcelDownloadDto input)
        {
            var downloadToken = await _downloadTokenCache.GetAsync(input.DownloadToken);
            if (downloadToken == null || input.DownloadToken != downloadToken.Token)
                throw new AbpAuthorizationException("Invalid download token: " + input.DownloadToken);

            var list = await _${entity_name_lower}Repository.GetListWithNavigationPropertiesAsync(${input_filter_call_args});
            var items = list.Select(item => new Dictionary<object, string>
            {
                ${excel_export_pairs}
            });

            var ms = new MemoryStream();
            await ms.SaveAsAsync(items);
            ms.Seek(0, SeekOrigin.Begin);

            return new RemoteStreamContent(ms, "${entity_plural}.xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
        }

        [Authorize(${domain_short}Permissions.${entity_plural}.Delete)]
        public virtual async Task DeleteByIdsAsync(List<Guid> ids)
        {
            await _${entity_name_lower}Repository.DeleteManyAsync(ids);
        }

        [Authorize(${domain_short}Permissions.${entity_plural}.Delete)]
        public virtual async Task DeleteAllAsync(Get${entity_plural}Input input)
        {
            await _${entity_name_lower}Repository.DeleteAllAsync(${input_filter_call_args});
        }

        public virtual async Task<DownloadTokenResultDto> GetDownloadTokenAsync()
        {
            var token = Guid.NewGuid().ToString("N");

            await _downloadTokenCache.SetAsync(
                token,
                new ${entity_name}DownloadTokenCacheItem { Token = token },
                new DistributedCacheEntryOptions { AbsoluteExpirationRelativeToNow = TimeSpan.FromSeconds(30) });

            return new DownloadTokenResultDto { Token = token };
        }
    }
}

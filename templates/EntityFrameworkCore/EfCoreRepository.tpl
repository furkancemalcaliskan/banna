using System;
using System.Collections.Generic;
using System.Linq;
using System.Linq.Dynamic.Core;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using Volo.Abp.Domain.Repositories.EntityFrameworkCore;
using Volo.Abp.EntityFrameworkCore;
using ${domain_name}.EntityFrameworkCore;
${nav_usings}

namespace ${namespace}
{
    public class EfCore${entity_name}Repository
        : EfCoreRepository<${domain_short}DbContext, ${entity_name}, Guid>, I${entity_name}Repository
    {
        public EfCore${entity_name}Repository(IDbContextProvider<${domain_short}DbContext> dbContextProvider)
            : base(dbContextProvider)
        {
        }

        public virtual async Task DeleteAllAsync(
            ${repository_filters}
            CancellationToken cancellationToken = default
        )
        {
            var query = await GetQueryForNavigationPropertiesAsync();
            query = ApplyFilter(query, ${filter_param_names});
            var ids = query.Select(x => x.${entity_name}.Id);
            await DeleteManyAsync(ids, cancellationToken: GetCancellationToken(cancellationToken));
        }

        public virtual async Task<${entity_name}WithNavigationProperties> GetWithNavigationPropertiesAsync(
            Guid id,
            CancellationToken cancellationToken = default
        )
        {
            var dbContext = await GetDbContextAsync();

            return (await GetDbSetAsync())
                .Where(x => x.Id == id)
                .Select(x => new ${entity_name}WithNavigationProperties
                {
                    ${entity_name} = x,
                    ${navigation_selects_single}
                })
                .FirstOrDefault();
        }

        public virtual async Task<List<${entity_name}WithNavigationProperties>> GetListWithNavigationPropertiesAsync(
            ${repository_filters}
            string? sorting = null,
            int maxResultCount = int.MaxValue,
            int skipCount = 0,
            CancellationToken cancellationToken = default
        )
        {
            var query = await GetQueryForNavigationPropertiesAsync();
            query = ApplyFilter(query, ${filter_param_names});
            query = query.OrderBy(string.IsNullOrWhiteSpace(sorting)
                ? ${entity_name}Consts.GetDefaultSorting(true)
                : sorting);
            return await query.PageBy(skipCount, maxResultCount).ToListAsync(cancellationToken);
        }

        protected virtual async Task<IQueryable<${entity_name}WithNavigationProperties>> GetQueryForNavigationPropertiesAsync()
        {
            var dbContext = await GetDbContextAsync();
            var ${entity_plural_lower} = await GetDbSetAsync();

            return from ${entity_name_lower} in ${entity_plural_lower}
                  ${navigation_joins}                   
                  select new ${entity_name}WithNavigationProperties
                  {
                      ${entity_name} = ${entity_name_lower},
                      ${navigation_selects_list}
                  };
        }

        protected virtual IQueryable<${entity_name}WithNavigationProperties> ApplyFilter(
            IQueryable<${entity_name}WithNavigationProperties> query,
            ${filter_method_params}
        )
        {
            return query
                    ${repository_nav_filters};
        }

        public virtual async Task<List<${entity_name}>> GetListAsync(
            ${repository_filters}
            string? sorting = null,
            int maxResultCount = int.MaxValue,
            int skipCount = 0,
            CancellationToken cancellationToken = default
        )
        {
            var q = ApplyFilter(await GetQueryableAsync(), ${filter_param_names});
            q = q.OrderBy(string.IsNullOrWhiteSpace(sorting)
                ? ${entity_name}Consts.GetDefaultSorting(false)
                : sorting);
            return await q.PageBy(skipCount, maxResultCount).ToListAsync(cancellationToken);
        }

        public virtual async Task<long> GetCountAsync(
            ${repository_filters}
            CancellationToken cancellationToken = default
        )
        {
            var query = await GetQueryForNavigationPropertiesAsync();
            query = ApplyFilter(query, ${filter_param_names});
            return await query.LongCountAsync(GetCancellationToken(cancellationToken));
        }

        protected virtual IQueryable<${entity_name}> ApplyFilter(
            IQueryable<${entity_name}> query,
            ${filter_method_params}
        )
        {
            return query
                    ${repository_entity_filters};
        }
    }
}

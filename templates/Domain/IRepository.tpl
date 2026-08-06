using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Volo.Abp.Domain.Repositories;

namespace ${namespace}
{
    public interface I${entity_name}Repository : IRepository<${entity_name}, Guid>
    {
        Task DeleteAllAsync(
          ${filter_method_params},
          CancellationToken cancellationToken = default
        );

        Task<${entity_name}WithNavigationProperties> GetWithNavigationPropertiesAsync(
            Guid id,
            CancellationToken cancellationToken = default
        );

        Task<List<${entity_name}WithNavigationProperties>> GetListWithNavigationPropertiesAsync(
            ${filter_method_params},
            string? sorting = null,
            int maxResultCount = int.MaxValue,
            int skipCount = 0,
            CancellationToken cancellationToken = default
        );

        Task<List<${entity_name}>> GetListAsync(
            ${filter_method_params},
            string? sorting = null,
            int maxResultCount = int.MaxValue,
            int skipCount = 0,
            CancellationToken cancellationToken = default
        );

        Task<long> GetCountAsync(
            ${filter_method_params},
            CancellationToken cancellationToken = default
        );
    }
}

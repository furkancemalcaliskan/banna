using System;
using System.Threading.Tasks;
using JetBrains.Annotations;
using Volo.Abp;
using Volo.Abp.Data;
using Volo.Abp.Domain.Services;

namespace ${namespace}
{
    public class ${entity_name}Manager : DomainService
    {
        protected I${entity_name}Repository _${entity_name_lower}Repository;

        public ${entity_name}Manager(I${entity_name}Repository ${entity_name_lower}Repository)
        {
            _${entity_name_lower}Repository = ${entity_name_lower}Repository;
        }

        public virtual async Task<${entity_name}> CreateAsync(
            ${manager_params}
        )
        {
            ${manager_checks}
            var ${entity_name_lower} = new ${entity_name}(
                GuidGenerator.Create(), ${manager_param_names}
            );

            return await _${entity_name_lower}Repository.InsertAsync(${entity_name_lower});
        }

        public virtual async Task<${entity_name}> UpdateAsync(
            Guid id,
            ${manager_params},
            [CanBeNull] string? concurrencyStamp = null
        )
        {
            ${manager_checks}
            var ${entity_name_lower} = await _${entity_name_lower}Repository.GetAsync(id);
            ${manager_assignments}
            ${entity_name_lower}.SetConcurrencyStampIfNotNull(concurrencyStamp);
            return await _${entity_name_lower}Repository.UpdateAsync(${entity_name_lower});
        }
    }
}

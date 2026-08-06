using System;
using Volo.Abp.Application.Dtos;
using Volo.Abp.Domain.Entities;

namespace ${namespace}
{
    public class ${entity_name}Dto : FullAuditedEntityDto<Guid>, IHasConcurrencyStamp
    {
        ${dto_props}
    }
}

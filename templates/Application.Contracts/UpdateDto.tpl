using System;
using System.ComponentModel.DataAnnotations;
using Volo.Abp.Domain.Entities;

namespace ${namespace}
{
    public class ${entity_name}UpdateDto : IHasConcurrencyStamp
    {
        ${update_dto_props}
    }
}

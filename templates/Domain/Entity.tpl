using System;
using Volo.Abp;
using Volo.Abp.Domain.Entities.Auditing;

namespace ${namespace}
{
    public class ${entity_name} : FullAuditedAggregateRoot<Guid>
    {
        ${fields}
        protected ${entity_name}() { }

        public ${entity_name}(Guid id${constructor_params})
        {
            Id = id;
            ${fields_assignments}
        }
    }
}

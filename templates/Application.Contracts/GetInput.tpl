using Volo.Abp.Application.Dtos;
using System;

namespace ${namespace}
{
    public class Get${entity_plural}Input : PagedAndSortedResultRequestDto
    {
        ${get_input_props}

        public Get${entity_plural}Input() { }
    }
}

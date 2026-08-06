using System;
${nav_usings}

namespace ${namespace}
{
    public class ${entity_name}WithNavigationPropertiesDto
    {
        public ${entity_name}Dto ${entity_name} { get; set; } = null!;
        ${with_nav_dto_props}
    }
}

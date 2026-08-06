using ${domain_name}.Shared;
using System;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Mvc;
using ${namespace};
${nav_usings}

namespace ${web_namespace}
{
    public class DetailModalModel : ${domain_short}PageModel
    {
        [HiddenInput]
        [BindProperty(SupportsGet = true)]
        public Guid Id { get; set; }

        [BindProperty]
        public ${entity_name}DetailViewModel ${entity_name} { get; set; }

        ${nav_dtos}

        protected I${entity_plural}AppService _${entity_name_lower}AppService;

        public DetailModalModel(I${entity_plural}AppService ${entity_name_lower}AppService)
        {
            _${entity_name_lower}AppService = ${entity_name_lower}AppService;
            ${entity_name} = new();
        }

        public virtual async Task OnGetAsync()
        {
            var ${entity_name_lower}WithNavigationPropertiesDto = await _${entity_name_lower}AppService.GetWithNavigationPropertiesAsync(Id);
            ${entity_name} = ObjectMapper.Map<${entity_name}Dto, ${entity_name}DetailViewModel>(${entity_name_lower}WithNavigationPropertiesDto.${entity_name});
            ${nav_dto_mappings}
        }
    }

    public class ${entity_name}DetailViewModel : ${entity_name}UpdateDto
    {
    }
}

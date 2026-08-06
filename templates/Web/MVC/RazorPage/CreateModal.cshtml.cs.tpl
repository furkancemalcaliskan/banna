using ${domain_name}.Shared;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Mvc;
using ${namespace};

namespace ${web_namespace}
{
    public class CreateModalModel : ${domain_short}PageModel
    {
        [BindProperty]
        public ${entity_name}CreateViewModel ${entity_name} { get; set; }

        protected I${entity_plural}AppService _${entity_name_lower}AppService;

        public CreateModalModel(I${entity_plural}AppService ${entity_name_lower}AppService)
        {
            _${entity_name_lower}AppService = ${entity_name_lower}AppService;
            ${entity_name} = new();
        }

        public virtual async Task OnGetAsync()
        {
            ${entity_name} = new ${entity_name}CreateViewModel();
            await Task.CompletedTask;
        }

        public virtual async Task<IActionResult> OnPostAsync()
        {
            await _${entity_name_lower}AppService.CreateAsync(ObjectMapper.Map<${entity_name}CreateViewModel, ${entity_name}CreateDto>(${entity_name}));
            return NoContent();
        }
    }

    public class ${entity_name}CreateViewModel : ${entity_name}CreateDto
    {
    }
}

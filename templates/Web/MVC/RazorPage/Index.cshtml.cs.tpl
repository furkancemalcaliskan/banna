using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Mvc.Rendering;
using Volo.Abp.Application.Dtos;
using Volo.Abp.AspNetCore.Mvc.UI.RazorPages;
using Volo.Abp.AspNetCore.Mvc.UI.Bootstrap.TagHelpers.Form;
using ${namespace};
using ${domain_name}.Shared;

namespace ${web_namespace}
{
    public class IndexModel : AbpPageModel
    {
        ${index_cs_nav_props}

        protected I${entity_plural}AppService _${entity_name_lower}AppService;

        public IndexModel(I${entity_plural}AppService ${entity_name_lower}AppService)
        {
            _${entity_name_lower}AppService = ${entity_name_lower}AppService;
        }

        public virtual async Task OnGetAsync()
        {
            ${index_cs_onget_nav_fill}
            await Task.CompletedTask;
        }
    }
}


using Volo.Abp.AspNetCore.Mvc.UI.RazorPages;
using Microsoft.AspNetCore.Mvc;

namespace ${web_namespace}
{
    public class CreateModel : AbpPageModel
    {
        public IActionResult OnGet()
        {
            if (CurrentUser.IsAuthenticated) return Page();
            return RedirectToPage("/Account/Login");
        }
    }
}

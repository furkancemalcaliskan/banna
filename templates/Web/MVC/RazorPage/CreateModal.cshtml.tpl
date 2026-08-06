@page
@using Microsoft.AspNetCore.Mvc.Localization
@using ${domain_name}.Localization
@using Volo.Abp.AspNetCore.Mvc.UI.Bootstrap.TagHelpers.Modal
@using ${namespace}
@using ${web_namespace}
@inject IHtmlLocalizer<${domain_short}Resource> L
@model CreateModalModel
@{ Layout = null; }

@section styles { <abp-style src="/Pages/${entity_plural}/CreateModal.css" /> }

<form data-ajaxForm="true" asp-page="/${entity_plural}/CreateModal" autocomplete="off">
  <abp-modal id="${entity_name}CreateModal" size="Large" centered="true">
    <abp-modal-header title="@L["New${entity_name}"].Value" class="border-1 border-bottom mb-3"></abp-modal-header>
    <abp-modal-body>
      ${web_form_fields_create}
    </abp-modal-body>
    <abp-modal-footer buttons="@(AbpModalButtons.Cancel|AbpModalButtons.Save)">
    </abp-modal-footer>
  </abp-modal>
</form>

@page
@using Microsoft.AspNetCore.Mvc.Localization
@using ${domain_name}.Localization
@inject IHtmlLocalizer<${domain_short}Resource> L
@{ Layout = null; }

@section styles { <abp-style src="/Pages/${entity_plural}/CreateModal.css" /> }
@section scripts { <abp-script src="/Pages/${entity_plural}/createModal.js" /> }

<abp-modal id="${entity_name}CreateModal" size="Large" centered="true">
  <abp-modal-body class="p-0">
    <script type="text/x-template" id="${entity_name_lower}-create-template">
      <div class="modal-content border-0">
        <div class="modal-header border-1 border-bottom mb-3">
          <h5 class="modal-title">@L["New${entity_name}"].Value</h5>
          <button type="button" class="btn-close" data-bs-dismiss="modal" @@click="close()"></button>
        </div>

        <div class="modal-body">
          ${web_form_fields_create_vue}
        </div>

        <div class="modal-footer">
          <button class="btn btn-outline-primary" type="button" data-bs-dismiss="modal" @@click="close()">
            @L["Cancel"].Value
          </button>
          <button class="btn btn-primary" type="button" :disabled="!canSave" @@click="save()">
            @L["Save"].Value
          </button>
        </div>
      </div>
    </script>

    <div id="${entity_name_lower}-create-app"></div>
  </abp-modal-body>
</abp-modal>

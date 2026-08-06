@page
@using Microsoft.AspNetCore.Mvc.Localization
@using ${domain_name}.Localization
@inject IHtmlLocalizer<${domain_short}Resource> L
@{ Layout = null; }

@section styles { <abp-style src="/Pages/${entity_plural}/DetailModal.css" /> }
@section scripts { <abp-script src="/Pages/${entity_plural}/detailModal.js" /> }

<abp-modal id="${entity_name}DetailModal" size="Large" centered="true">
  <abp-modal-body class="p-0">
    <script type="text/x-template" id="${entity_name_lower}-detail-template">
      <div class="modal-content border-0">
        <div class="modal-header border-1 border-bottom mb-3">
          <h5 class="modal-title">@L["Detail"].Value</h5>
          <button type="button" class="btn-close" data-bs-dismiss="modal" @@click="close()"></button>
        </div>

        <div class="modal-body">
          ${web_form_fields_detail_vue}
        </div>

        <div class="modal-footer">
          <button class="btn btn-secondary" type="button" data-bs-dismiss="modal" @@click="close()">
            @L["Close"].Value
          </button>
        </div>
      </div>
    </script>

    <div id="${entity_name_lower}-detail-app"></div>
  </abp-modal-body>
</abp-modal>


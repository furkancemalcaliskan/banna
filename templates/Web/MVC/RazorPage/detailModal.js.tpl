var abp = abp || {};
abp.modals.${entity_camel}Detail = function () {
  var initModal = function (publicApi, args) {
    publicApi.onOpen(function () {
      ${web_js_select2_detail}
    });
  };
  return { initModal: initModal };
};

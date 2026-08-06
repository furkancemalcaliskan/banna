var abp = abp || {};
abp.modals.${entity_camel}Edit = function () {
  var initModal = function (publicApi, args) {
    publicApi.onOpen(function () {
      ${web_js_select2_edit}
    });
  };
  return { initModal: initModal };
};

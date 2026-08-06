var abp = abp || {};
abp.modals.${entity_camel}Create = function () {
  var initModal = function (publicApi, args) {
    publicApi.onOpen(function () {
      ${web_js_select2_create}
    });
  };
  return { initModal: initModal };
};

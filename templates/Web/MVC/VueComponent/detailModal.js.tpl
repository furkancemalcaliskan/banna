var abp = abp || {};

abp.modals.${entity_name_lower}Detail = abp.vueModal({
  modalId: "${entity_name}DetailModal",
  appElementId: "${entity_name_lower}-detail-app",
  templateId: "#${entity_name_lower}-detail-template",
  components: { "v-select": window["vue-select"] },
  setup({ ready, publicApi, args, Vue }) {
    const { ref, reactive, onMounted } = Vue;
    const l = abp.localization.getResource("${domain_short}");
    const ${entity_name_lower}Service = window.${namespace_lower}.${entity_plural_lower};
    const id = args?.id;

    const model = reactive({
      ${vue_detail_model_fields}
    });

    ${vue_detail_support_refs}

    async function load() {
      const dto = await ${entity_name_lower}Service.getWithNavigationProperties(id);
      ${vue_detail_load_mapping}
    }

    function close(){ publicApi.close(); }

    onMounted(async () => {
      try {
        await load();
      } finally {
        ready();
      }
    });

    return { model, close${vue_detail_return_refs} };
  }
});

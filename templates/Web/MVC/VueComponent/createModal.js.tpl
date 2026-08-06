var abp = abp || {};

abp.modals.${entity_name_lower}Create = abp.vueModal({
  modalId: "${entity_name}CreateModal",
  appElementId: "${entity_name_lower}-create-app",
  templateId: "#${entity_name_lower}-create-template",
  components: { "v-select": window["vue-select"] },
  setup({ ready, publicApi, Vue }) {
    const { ref, reactive, computed, onMounted } = Vue;
    const l = abp.localization.getResource("${domain_short}");
    const ${entity_name_lower}Service = window.${namespace_lower}.${entity_plural_lower};

    const model = reactive({
      ${vue_create_model_fields}
    });

    ${vue_create_support_refs}

    ${vue_create_lookup_functions}

    const isSaving = ref(false);
    const canSave = computed(() => {
      return !isSaving.value && (${vue_create_cansave_expression});
    });

    async function save() {
      if (!canSave.value) return;
      isSaving.value = true;
      try {
        await ${entity_name_lower}Service.create(${vue_create_payload});
        abp.notify.success(l("SuccessfullySaved"));
        publicApi.setResult && publicApi.setResult({});
        publicApi.close();
      } finally {
        isSaving.value = false;
      }
    }
    function close(){ publicApi.close(); }

    onMounted(async () => {
      try {
        ${vue_create_onmounted}
      } finally {
        ready();
      }
    });

    return { model, canSave, save, close${vue_create_return_refs} };
  }
});

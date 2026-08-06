var abp = abp || {};

abp.modals.${entity_name_lower}Edit = abp.vueModal({
  modalId: "${entity_name}EditModal",
  appElementId: "${entity_name_lower}-edit-app",
  templateId: "#${entity_name_lower}-edit-template",
  components: { "v-select": window["vue-select"] },
  setup({ ready, publicApi, args, Vue }) {
    const { ref, reactive, computed, onMounted } = Vue;
    const l = abp.localization.getResource("${domain_short}");
    const ${entity_name_lower}Service = window.${namespace_lower}.${entity_plural_lower};
    const id = args?.id;

    const model = reactive({
      ${vue_edit_model_fields}
    });

    ${vue_edit_support_refs}
    ${vue_edit_lookup_functions}

    const isSaving = ref(false);
    const canSave = computed(() => !isSaving.value && (${vue_edit_cansave_expression}));

    async function load() {
      const dto = await ${entity_name_lower}Service.getWithNavigationProperties(id);
      ${vue_edit_load_mapping}
      ${vue_edit_prime_lookups_if_needed}
    }

    async function save() {
      if (!canSave.value) return;
      isSaving.value = true;
      try {
        await ${entity_name_lower}Service.update(id, ${vue_edit_payload});
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
        await load();
      } finally {
        ready();
      }
    });

    return { model, canSave, save, close${vue_edit_return_refs} };
  }
});

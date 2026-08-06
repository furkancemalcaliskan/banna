var abp = abp || {};

(function() {
  const { createApp } = Vue;

  function initDeferOpen(modalId, mountFn, publicApi, args) {
    const element = document.getElementById(modalId);
    if (!element) return;

    const handler = async (e) => {
      e.preventDefault();
      element.removeEventListener("show.bs.modal", handler);
      try {
        const { whenReady } = mountFn(publicApi, args);
        await whenReady;
        const inst = bootstrap.Modal.getOrCreateInstance(element);
        inst.show();
      } catch (error) {
        abp.log?.error?.(error);
        publicApi.close?.();
      }
    };

    element.addEventListener("show.bs.modal", handler);
  }

  function mountVueApp({ appElementId, templateId, components, setup, publicApi, args }) {
    const mountElement = document.getElementById(appElementId);
    if (!mountElement) {
      throw new Error(`Vue modal mount element not found: ${appElementId}`);
    }

    let resolveReady;
    const whenReady = new Promise((r) => (resolveReady = r));

    const app = createApp({
      template: templateId,
      components: components || {},
      setup() {
        const ctx = {
          ready: () => resolveReady(),
          publicApi,
          args,
          Vue
        };
        return setup(ctx);
      }
    });

    app.mount(mountElement);
    return { whenReady };
  }

  abp.vueModal = function modalFactory(def) {
    return function() {
      return {
        initModal(publicApi, args) {
          initDeferOpen(
            def.modalId,
            (pub, a) => mountVueApp({
              appElementId: def.appElementId,
              templateId: def.templateId,
              components: def.components,
              setup: def.setup,
              publicApi: pub,
              args: a
            }),
            publicApi,
            args
          );
        }
      };
    };
  };
})();

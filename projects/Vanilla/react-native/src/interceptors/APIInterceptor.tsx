import i18n from '../utils/i18n';
import api from '../api/API';
import { refresh } from '../api/AccountAPI';
import LoadingActions from '../store/actions/LoadingActions';
import AppActions from '../store/actions/AppActions';
import PersistentStorageActions from '../store/actions/PersistentStorageActions';
import React from 'react';
import { Animated, Dimensions, PanResponder, StyleSheet } from 'react-native';
import { Toast, ToastTitle, ToastDescription, VStack } from '@gluestack-ui/themed';

const DISMISS_THRESHOLD = 80;
const DISMISS_DURATION = 180;

function SwipeableToast({ id, toast, title, message }) {
  const translate = React.useRef(new Animated.ValueXY({ x: 0, y: 0 })).current;
  const windowSize = React.useRef(Dimensions.get('window')).current;

  const resetPosition = React.useCallback(() => {
    Animated.spring(translate, {
      toValue: { x: 0, y: 0 },
      useNativeDriver: false,
    }).start();
  }, [translate]);

  const closeToast = React.useCallback(() => {
    if (toast?.close) {
      toast.close(id);
      return;
    }
    if (toast?.hide) {
      toast.hide(id);
    }
  }, [id, toast]);

  const dismiss = React.useCallback(
    (toValue) => {
      Animated.timing(translate, {
        toValue,
        duration: DISMISS_DURATION,
        useNativeDriver: false,
      }).start(closeToast);
    },
    [closeToast, translate],
  );

  const panResponder = React.useRef(
    PanResponder.create({
      onMoveShouldSetPanResponder: (_, gesture) =>
        Math.abs(gesture.dx) > 6 || Math.abs(gesture.dy) > 6,
      onPanResponderMove: Animated.event([null, { dx: translate.x, dy: translate.y }], {
        useNativeDriver: false,
      }),
      onPanResponderRelease: (_, gesture) => {
        const { dx, dy } = gesture;
        if (Math.abs(dx) > DISMISS_THRESHOLD || Math.abs(dy) > DISMISS_THRESHOLD) {
          if (Math.abs(dx) >= Math.abs(dy)) {
            dismiss({ x: dx > 0 ? windowSize.width : -windowSize.width, y: dy * 0.2 });
          } else {
            dismiss({ x: dx * 0.2, y: dy > 0 ? windowSize.height : -windowSize.height });
          }
          return;
        }
        resetPosition();
      },
      onPanResponderTerminate: resetPosition,
    }),
  ).current;

  return (
    <Animated.View
      style={[styles.toastWrapper, { transform: translate.getTranslateTransform() }]}
      {...panResponder.panHandlers}
    >
      <Toast
        nativeID={"toast-" + id}
        action="error"
        variant="solid"
        bg="$themeSurfaceBg"
        borderWidth={1}
        borderColor="$themeBorder"
        borderRadius="$3xl"
        px="$4"
        py="$3"
        _dark={{ bg: '$themeSurfaceBgDark', borderColor: '$themeBorderDark' }}
        style={styles.toastShadow}
      >
        <VStack space="xs">
          <ToastTitle>{title}</ToastTitle>
          {message ? <ToastDescription>{message}</ToastDescription> : null}
        </VStack>
      </Toast>
    </Animated.View>
  );
}

const styles = StyleSheet.create({
  toastWrapper: {
    paddingHorizontal: 12,
  },
  toastShadow: {
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 8 },
    shadowOpacity: 0.18,
    shadowRadius: 18,
    elevation: 6,
  },
});

let globalToast = null;

export function setAPIInterceptorToast(toast) {
  globalToast = toast;
}

export function initAPIInterceptor(store) {
  api.interceptors.request.use(
    async request => {
      const {
        persistentStorage: { token, language, tenant },
      } = store.getState();

      if (!request.headers.Authorization && token && token.access_token) {
        request.headers.Authorization = `${token.token_type} ${token.access_token}`;
      }

      if (!request.headers['Content-Type']) {
        request.headers['Content-Type'] = 'application/json';
      }

      if (!request.headers['Accept-Language'] && language) {
        request.headers['Accept-Language'] = language;
      }

      if (!request.headers.__tenant && tenant && tenant.tenantId) {
        request.headers.__tenant = tenant.tenantId;
      }

      return request;
    },
    error => console.error(error),
  );

  api.interceptors.response.use(
    response => response,
    async error => {
      store.dispatch(LoadingActions.clear());
      const errorRes = error.response;
      const originalRequest = error.config || {};
      if (errorRes) {
        if (errorRes.status === 401 && !originalRequest._retry) {
          const stateToken = store.getState()?.persistentStorage?.token || {};
          const refreshToken = stateToken.refresh_token;
          if (refreshToken) {
            originalRequest._retry = true;
            try {
              const data = await refresh(refreshToken);
              const expireTime = new Date().valueOf() + data.expires_in * 1000;
              const nextToken = {
                ...stateToken,
                ...data,
                refresh_token: data.refresh_token || refreshToken,
                expire_time: expireTime,
                scope: undefined,
              };
              store.dispatch(PersistentStorageActions.setToken(nextToken));
              store.dispatch(AppActions.fetchAppConfigAsync({}));
              originalRequest.headers = {
                ...(originalRequest.headers || {}),
                Authorization: `${nextToken.token_type} ${nextToken.access_token}`,
              };
              return api(originalRequest);
            } catch (refreshError) {
              store.dispatch(PersistentStorageActions.setToken({}));
            }
          } else if (errorRes.headers._abperrorformat) {
            store.dispatch(PersistentStorageActions.setToken({}));
          }
        } else if (errorRes.headers._abperrorformat && errorRes.status === 401) {
          store.dispatch(PersistentStorageActions.setToken({}));
        }

        showError({ error: errorRes.data.error || {}, status: errorRes.status }, globalToast);
      } else {
        if (globalToast) {
          const isNetworkError =
            error?.message === 'Network Error' ||
            error?.code === 'ERR_NETWORK' ||
            error?.message?.toLowerCase?.().includes('network');
          const title = isNetworkError
            ? i18n.t('App::NetworkErrorTitle')
            : i18n.t('AbpAccount::DefaultErrorMessage');
          const message = isNetworkError
            ? i18n.t('App::NetworkErrorMessage')
            : null;

          globalToast.show({
            placement: 'top',
            render: ({ id }) => (
              <SwipeableToast
                id={id}
                toast={globalToast}
                title={title}
                message={message}
              />
            ),
          });
        }
      }

      return Promise.reject(error);
    },
  );
}

function showError({ error = {}, status }, toast) {
  let message = '';
  let title = i18n.t('AbpAccount::DefaultErrorMessage');

  if (typeof error === 'string') {
    message = error;
  } else if (error.details) {
    message = error.details;
    title = error.message;
  } else if (error.message) {
    message = error.message;
  } else {
    switch (status) {
      case 401:
        title = i18n.t('AbpAccount::DefaultErrorMessage401');
        message = i18n.t('AbpAccount::DefaultErrorMessage401Detail');
        break;
      case 403:
        title = i18n.t('AbpAccount::DefaultErrorMessage403');
        message = i18n.t('AbpAccount::DefaultErrorMessage403Detail');
        break;
      case 404:
        title = i18n.t('AbpAccount::DefaultErrorMessage404');
        message = i18n.t('AbpAccount::DefaultErrorMessage404Detail');
        break;
      case 500:
        title = i18n.t('AbpAccount::500Message');
        message = i18n.t('AbpAccount::InternalServerErrorMessage');
        break;
      default:
        break;
    }
  }

  if (toast) {
    toast.show({
      placement: 'top',
      render: ({ id }) => (
        <SwipeableToast id={id} toast={toast} title={title} message={message} />
      ),
    });
  }
}

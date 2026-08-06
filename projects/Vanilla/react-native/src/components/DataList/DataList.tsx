import { Ionicons } from '@expo/vector-icons';
import { useFocusEffect } from '@react-navigation/native';
import { useHeaderHeight } from '@react-navigation/elements';
import i18n from '../../utils/i18n';
import {
  Box,
  Center,
  FlatList,
  Icon,
  Input,
  InputField,
  InputSlot,
  Spinner,
  Text,
} from '@gluestack-ui/themed';
import React, { forwardRef, useCallback, useEffect, useState } from 'react';
import { FlatListProps as RNFlatListProps, StyleSheet, View } from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import LoadingActions from '../../store/actions/LoadingActions';
import { debounce } from '../../utils/Debounce';
import { connectToRedux } from '../../utils/ReduxConnect';
import LoadingButton from '../LoadingButton/LoadingButton';
import { useInputColors } from '../../theme';

type FetchResponse<T> = {
  items: T[];
  totalCount: number;
};

type DataListProps<T> = RNFlatListProps<T> & {
  fetchFn: (args: { filter: string; maxResultCount: number; skipCount: number }) => Promise<FetchResponse<T>>;
  render: (...args: any[]) => React.ReactElement;
  maxResultCount?: number;
  debounceTime?: number;
  trigger?: any;
  refreshing?: boolean;
  onRefresh?: () => void;
  onRefreshComplete?: () => void;
};

function DataList<T = any>({
  navigation,
  fetchFn,
  render,
  maxResultCount = 15,
  debounceTime = 350,
  trigger = null,
  refreshing = undefined,
  onRefresh = undefined,
  onRefreshComplete = undefined,
  ...props
}: DataListProps<T>) {
  const [records, setRecords] = useState<T[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [loading, setLoading] = useState(false);
  const [searchLoading, setSearchLoading] = useState(false);
  const [buttonLoading, setButtonLoading] = useState(false);
  const [skipCount, setSkipCount] = useState(0);
  const [filter, setFilter] = useState('');
  const headerHeight = useHeaderHeight();
  const insets = useSafeAreaInsets();
  const topInset = headerHeight > 0 ? 0 : insets.top || 0;
  const inputColors = useInputColors();

  const fetch = (skip = 0, isRefreshingActive = true) => {
    if (isRefreshingActive) setLoading(true);
    return fetchFn({ filter, maxResultCount, skipCount: skip })
      .then(({ items, totalCount: total }) => {
        setTotalCount(total);
        setRecords(skip ? [...records, ...items] : items);
        setSkipCount(skip);
      })
      .finally(() => {
        if (isRefreshingActive) setLoading(false);
      });
  };

  const refresh = () =>
    fetch(0, true).finally(() => {
      if (onRefreshComplete) {
        onRefreshComplete();
      }
    });

  const fetchPartial = () => {
    if (loading || records.length === totalCount) return;

    setButtonLoading(true);
    fetch(skipCount + maxResultCount, false).finally(() =>
      setButtonLoading(false)
    );
  };

  useFocusEffect(
    useCallback(() => {
      setSkipCount(0);
      fetch(0, false);
    }, [])
  );

  useEffect(() => {
    function searchFetch() {
      setSearchLoading(true);
      return fetch(0, false).finally(() =>
        setTimeout(() => setSearchLoading(false), 150)
      );
    }
    debounce(searchFetch, debounceTime)();
  }, [filter]);

  useEffect(() => {
    if (trigger === null || trigger === undefined) {
      return;
    }
    refresh();
  }, [trigger]);

  return (
    <Center flex={1}>
      <Box flex={1} w="$full" px="$4" mt="$2" style={{ paddingTop: topInset }}>
        <Input
          size="lg"
          bg={inputColors.bg}
          borderWidth={1}
          borderColor={inputColors.borderColor}
          borderRadius="$lg"
          mb="$4"
        >
          <InputField
            placeholder={i18n.t('AbpUi::PagerSearch')}
            returnKeyType="done"
            value={filter}
            onChangeText={setFilter}
            color={inputColors.textColor}
            placeholderTextColor={inputColors.placeholderColor}
            selectionColor={inputColors.textColor}
          />
          <InputSlot pr="$3">
            {searchLoading ? (
              <Spinner color={inputColors.iconColor} size="small" />
            ) : (
              <Icon
                as={Ionicons}
                name="search"
                size="md"
                color={inputColors.iconColor}
              />
            )}
          </InputSlot>
        </Input>
        <FlatList
          flex={1}
          mt="$2"
          contentContainerStyle={{
            flexGrow: 1,
            paddingBottom: Math.max(insets.bottom, 20),
          }}
          alwaysBounceVertical
          bounces
          data={records}
          refreshing={refreshing}
          onRefresh={onRefresh}
          renderItem={(...args) => (
            <>
              {render(...args)}
              {args.index + 1 === skipCount + maxResultCount &&
                totalCount > records.length ? (
                <View
                  style={{ justifyContent: 'center', alignItems: 'center', marginTop: 10 }}
                >
                  <LoadingButton
                    loading={buttonLoading}
                    onPress={() => fetchPartial()}
                  >
                    <Text>{i18n.t('AbpUi::LoadMore')}</Text>
                  </LoadingButton>
                </View>
              ) : null}
            </>
          )}
          {...props}
        />
      </Box>
    </Center>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1 },
  list: {},
});

const Forwarded = forwardRef<any, DataListProps<any>>((props, ref) => (
  <DataList {...props} forwardedRef={ref} />
));

export default connectToRedux({
  component: Forwarded,
  dispatchProps: {
    startLoading: LoadingActions.start,
    stopLoading: LoadingActions.stop,
  },
});

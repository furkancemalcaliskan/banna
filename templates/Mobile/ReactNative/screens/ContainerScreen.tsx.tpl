import React from 'react';
import i18n from '../../utils/i18n';
import { useSelector } from 'react-redux';
import { Box, HStack, Pressable, Text } from '@gluestack-ui/themed';

import __entity_plural__Screen from './__entity_plural__Screen';
import { createAppConfigSelector } from '../../store/selectors/AppSelectors';
import { useSurfaceColors } from '../../theme';

type ContainerScreenProps = {
  navigation: any;
};

const scenes = (nav: any) => ({
  __entity_plural_lower__: () => <__entity_plural__Screen navigation={nav} />,
__mobile_tab_scenes__
});

function __entity_plural__MainScreen({ navigation }: ContainerScreenProps) {
  const [index, setIndex] = React.useState(0);
  const [routes, setRoutes] = React.useState<any[]>([]);
  const surface = useSurfaceColors();

  const currentUser = useSelector(createAppConfigSelector())?.currentUser;
  const policies = useSelector(createAppConfigSelector())?.auth?.grantedPolicies;

  const sceneMap = React.useMemo(() => scenes(navigation), [navigation]);

  React.useEffect(() => {
    if (!currentUser?.isAuthenticated || !policies) {
      setRoutes([]);
      return;
    }

    const nextRoutes = [];

    if (!!policies['__domain_short__.__entity_plural__']) {
      nextRoutes.push({
        key: '__entity_plural_lower__',
        title: i18n.t('__domain_short__::Menu:__entity_plural__'),
      });
    }

__mobile_tab_route_builder__

    setRoutes([...nextRoutes]);
  }, [Object.keys(policies || {}).filter((key) => key.startsWith('__domain_short__')).length]);

  if (!routes.length) return null;

  const activeRoute = routes[index];
  const renderActive = activeRoute ? sceneMap[activeRoute.key] : null;

  return (
    <Box flex={1}>
      <HStack px="$3" py="$2" space="sm" justifyContent="flex-start">
        {routes.map((route, idx) => (
          <Pressable
            key={route.key}
            px="$3"
            py="$2"
            borderRadius="$lg"
            bg={idx === index ? surface.activeHighlight : 'transparent'}
            borderWidth={idx === index ? 1 : 0}
            borderColor={surface.borderColor}
            onPress={() => setIndex(idx)}
          >
            <Text
              fontWeight={idx === index ? '$bold' : '$normal'}
              color={idx === index ? surface.primaryText : surface.textMuted}
            >
              {route.title}
            </Text>
          </Pressable>
        ))}
      </HStack>
      <Box flex={1}>
        {renderActive ? renderActive() : null}
      </Box>
    </Box>
  );
}

export default __entity_plural__MainScreen;

import React from 'react';
import HomeScreen from '../screens/Home/HomeScreen';
import { createAppStackNavigator, getStackScreenOptions } from './StackFactory';
import { useAppSafeAreaTop } from '../theme';

const Stack = createAppStackNavigator();

export default function HomeStackNavigator() {
  const safeAreaTop = useAppSafeAreaTop();

  return (
    <Stack.Navigator
      initialRouteName="Home"
      screenOptions={getStackScreenOptions({ headerStatusBarHeight: safeAreaTop })}
    >
      <Stack.Screen
        name="Home"
        component={HomeScreen}
        options={{header: () => null}}
      />
    </Stack.Navigator>
  );
}

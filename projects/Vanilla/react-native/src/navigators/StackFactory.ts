import {
  createStackNavigator,
  CardStyleInterpolators,
  HeaderStyleInterpolators,
} from '@react-navigation/stack';

export const createAppStackNavigator = () => createStackNavigator();

export const getStackScreenOptions = (options: { headerStatusBarHeight?: number } = {}) => ({
  headerStyleInterpolator: HeaderStyleInterpolators.forUIKit,
  cardStyleInterpolator: CardStyleInterpolators.forHorizontalIOS,
  headerStatusBarHeight: options.headerStatusBarHeight,
});

import { connect } from 'react-redux';

type ConnectParams = {
  component: React.ComponentType<any>;
  stateProps?: (state: any) => Record<string, any>;
  dispatchProps?: Record<string, any>;
};

export function connectToRedux({ component, stateProps = () => ({}), dispatchProps = {} }: ConnectParams) {
  const mapStateToProps = stateProps;

  const mapDispatchToProps = dispatchProps;

  return connect(mapStateToProps, mapDispatchToProps)(component);
}

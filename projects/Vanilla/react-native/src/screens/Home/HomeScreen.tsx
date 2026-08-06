import i18n from '../../utils/i18n';
import { Center, Heading, Text } from '@gluestack-ui/themed';
import Card from '../../components/Layout/Card';
import React, { useContext } from 'react';
import { StyleSheet } from 'react-native';
import { LocalizationContext } from '../../contexts/LocalizationContext';

function HomeScreen() {
  const localization = useContext(LocalizationContext);
  const t = localization?.t || i18n.t;

  return (
    <Center flex={1} px="$4">
      <Card w="$full" p="$6" alignItems="center">
        <Heading
          style={styles.centeredText}
          size="xl"
        >
          {t('::Welcome')}
        </Heading>
        <Text style={styles.centeredText} color="$textDark500" _dark={{ color: '$textDarkMuted' }} mt="$2">
          {t('::LongWelcomeMessage')}
        </Text>
      </Card>
    </Center>
  );
}

const styles = StyleSheet.create({
  centeredText: {
    textAlign: 'center',
    marginBottom: 5
  },
});

export default HomeScreen;

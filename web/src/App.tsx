import {
  ChakraProvider,
  Container,
  Heading,
  Tab,
  TabList,
  Tabs,
  TabPanels,
  TabPanel,
} from '@chakra-ui/react'
import React from 'react'
import { CreatePool } from './views/createPool'
import { Swap } from './views/swap'

const App = () => {
  return (
    <ChakraProvider>
      <Container maxW='420px' p={3}>
        <Heading as='h1' mb={3}>
          Simple Token Pool
        </Heading>

        <Tabs>
          <TabList>
            <Tab>Create Pool</Tab>
            <Tab>Swap</Tab>
          </TabList>

          <TabPanels>
            <TabPanel>
              <CreatePool />
            </TabPanel>
            <TabPanel>
              <Swap />
            </TabPanel>
          </TabPanels>
        </Tabs>
      </Container>
    </ChakraProvider>
  )
}

export default App

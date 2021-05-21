import { Connection } from '@solana/web3.js'
import { useMemo } from 'react'

export const useConnection = (endpoint: string) => {
  const connection = useMemo(() => new Connection(endpoint, 'recent'), [endpoint])
  return connection
}

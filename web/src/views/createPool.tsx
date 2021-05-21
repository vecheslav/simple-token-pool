import { Button, FormControl, FormLabel, Input } from '@chakra-ui/react'
import { Keypair, PublicKey } from '@solana/web3.js'
import React from 'react'
import { useForm } from 'react-hook-form'
import { config } from '../config'
import { createPool } from '../helpers/pool'
import { useConnection } from '../hooks/useConnectin'
import { secretKey } from '../secret'

type FormData = {
  bankMint: string
}

export const CreatePool = () => {
  const connection = useConnection(config.endpoint)
  const {
    handleSubmit,
    register,
    formState: { isSubmitting },
  } = useForm<FormData>()
  const payer = Keypair.fromSecretKey(secretKey)

  const onTestCreatePool = handleSubmit(async ({ bankMint }) => {
    console.log('Creting new pool...')
    console.log(`Bank mint: ${bankMint}\nPayer: ${payer.publicKey}`)

    const res = await createPool(connection, payer, new PublicKey(bankMint))
    console.log(res)
  })

  return (
    <>
      <form onSubmit={onTestCreatePool}>
        <FormControl>
          <FormLabel>Bank Mint pubkey</FormLabel>
          <Input
            placeholder='Pubkey'
            defaultValue='4FUUh8mCT1kPNi2GFZJYRuC4pi3rNW6EY3NsBk6RKrpn'
            {...register('bankMint', { required: true })}
          />
        </FormControl>
        <Button type='submit' size='lg' width='100%' mt={4} isLoading={isSubmitting}>
          Continue
        </Button>
      </form>
    </>
  )
}

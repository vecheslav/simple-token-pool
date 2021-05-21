import { Button, FormControl, FormLabel, Input } from '@chakra-ui/react'
import { Keypair, PublicKey } from '@solana/web3.js'
import React from 'react'
import { useForm } from 'react-hook-form'
import { config } from '../config'
import { swap } from '../helpers/pool'
import { useConnection } from '../hooks/useConnectin'
import { secretKey } from '../secret'

type FormData = {
  pool: string
  sender: string
  recipient: string
  amount: string
}

export const Swap = () => {
  const connection = useConnection(config.endpoint)
  const {
    handleSubmit,
    register,
    formState: { isSubmitting },
  } = useForm<FormData>()
  const payer = Keypair.fromSecretKey(secretKey)

  const onTestSwap = handleSubmit(async ({ pool, sender, recipient, amount }) => {
    console.log('Swap...')

    const amountFloat = parseFloat(amount)
    console.log(amountFloat)

    await swap(
      connection,
      payer,
      new PublicKey(pool),
      new PublicKey(sender),
      new PublicKey(recipient),
      amountFloat,
    )
  })

  return (
    <>
      <form onSubmit={onTestSwap}>
        <FormControl>
          <FormLabel>Pool pubkey</FormLabel>
          <Input placeholder='Pubkey' {...register('pool', { required: true })} />
        </FormControl>
        <FormControl mt={3}>
          <FormLabel>Sender token pubkey</FormLabel>
          <Input placeholder='Pubkey' {...register('sender', { required: true })} />
        </FormControl>
        <FormControl mt={3}>
          <FormLabel>Recipient token pubkey</FormLabel>
          <Input placeholder='Pubkey' {...register('recipient', { required: true })} />
        </FormControl>
        <FormControl mt={3}>
          <FormLabel>Pool pubkey</FormLabel>
          <Input placeholder='100' {...register('amount', { required: true })} />
        </FormControl>
        <Button type='submit' size='lg' width='100%' mt={4} isLoading={isSubmitting}>
          Continue
        </Button>
      </form>
    </>
  )
}

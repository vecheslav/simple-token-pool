import { AccountLayout, MintLayout, TOKEN_PROGRAM_ID } from '@solana/spl-token'
import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
} from '@solana/web3.js'
import { config } from '../config'
import * as Instruction from './instruction'
import { PoolLayout } from './layout'

export const getPoolAccount = async (connection: Connection, pubkey: PublicKey) => {
  const info = await connection.getAccountInfo(pubkey)
  return info && PoolLayout.decode(Buffer.from(info.data))
}

export const createPool = async (
  connection: Connection,
  payer: Keypair,
  bankMintPubkey: PublicKey,
): Promise<{
  poolPublicKey: string
  poolMintPublicKey: string
  bankPublicKey: string
}> => {
  const pool = Keypair.generate()
  const poolMint = Keypair.generate()
  const bank = Keypair.generate()

  try {
    const poolSpace = PoolLayout.span
    const poolRent = await connection.getMinimumBalanceForRentExemption(poolSpace)

    const poolMintSpace = MintLayout.span
    const poolMintRent = await connection.getMinimumBalanceForRentExemption(poolMintSpace)

    const bankSpace = AccountLayout.span
    const bankRent = await connection.getMinimumBalanceForRentExemption(bankSpace)

    const [authority] = await PublicKey.findProgramAddress(
      [pool.publicKey.toBuffer()],
      new PublicKey(config.programId),
    )

    const tx = new Transaction()
      .add(
        SystemProgram.createAccount({
          fromPubkey: payer.publicKey,
          newAccountPubkey: pool.publicKey,
          lamports: poolRent,
          space: poolSpace,
          programId: new PublicKey(config.programId),
        }),
      )
      .add(
        SystemProgram.createAccount({
          fromPubkey: payer.publicKey,
          newAccountPubkey: poolMint.publicKey,
          lamports: poolMintRent,
          space: poolMintSpace,
          programId: TOKEN_PROGRAM_ID,
        }),
      )
      .add(
        SystemProgram.createAccount({
          fromPubkey: payer.publicKey,
          newAccountPubkey: bank.publicKey,
          lamports: bankRent,
          space: bankSpace,
          programId: TOKEN_PROGRAM_ID,
        }),
      )
      .add(
        Instruction.initialize({
          pool: pool.publicKey,
          authority,
          bankMint: bankMintPubkey,
          poolMint: poolMint.publicKey,
          bank: bank.publicKey,
        }),
      )

    const signature = await sendAndConfirmTransaction(connection, tx, [payer, pool, poolMint, bank])
    console.log(`Signature: ${signature}`)
  } catch (err) {
    console.error(err)
  }

  return {
    poolPublicKey: pool.publicKey.toString(),
    poolMintPublicKey: poolMint.publicKey.toString(),
    bankPublicKey: bank.publicKey.toString(),
  }
}

export const swap = async (
  connection: Connection,
  payer: Keypair,
  poolPubkey: PublicKey,
  senderToken: PublicKey,
  recipientToken: PublicKey,
  amountIn: number, // TODO: fix to bignumber "number | u64"
) => {
  try {
    const poolInfo = await getPoolAccount(connection, poolPubkey)
    if (!poolInfo) {
      throw new Error("Pool doesn't exists")
    }

    const poolMintPubkey = new PublicKey(poolInfo.pool_mint)
    const bankPubkey = new PublicKey(poolInfo.bank)

    const [poolAuthority] = await PublicKey.findProgramAddress(
      [poolPubkey.toBuffer()],
      new PublicKey(config.programId),
    )

    const tx = new Transaction().add(
      Instruction.swap({
        pool: poolPubkey,
        poolAuthority,
        userTransferAuthority: payer.publicKey,
        poolMint: poolMintPubkey,
        bank: bankPubkey,
        sender: senderToken,
        recipient: recipientToken,
        amountIn,
      }),
    )

    const signature = await sendAndConfirmTransaction(connection, tx, [payer])
    console.log(`Signature: ${signature}`)
  } catch (err) {
    console.error(err)
  }
}

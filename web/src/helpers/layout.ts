import BufferLayout from 'buffer-layout'
import * as BaseLayout from './baseLayout'

export const InstructionLayout = BufferLayout.u8('instruction')

export const PoolLayout = BufferLayout.struct([
  BufferLayout.u8('version'),
  BaseLayout.PublicKey('authority'),
  BufferLayout.u8('bump_seed'),
  BaseLayout.PublicKey('bank_mint'),
  BaseLayout.PublicKey('pool_mint'),
  BaseLayout.PublicKey('bank'),
])

export const PoolInsructionLayouts = {
  Initialize: {
    index: 0,
    layout: BufferLayout.struct([InstructionLayout]),
  },
  Swap: {
    index: 1,
    layout: BufferLayout.struct([InstructionLayout, BaseLayout.Uint64('amountIn')]),
  },
}

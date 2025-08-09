import test from 'ava'

import { simpleCallbackTest } from '../index'

test('simpleCallbackTest', (t) => {
  const fixture = 42
  simpleCallbackTest(fixture, (arg) => {
    t.is(arg, fixture * 2)
    return arg
  })
})

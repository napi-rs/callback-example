import { calculateSalary } from './index.js'

const salary = calculateSalary(1000, (baseAmount, tax, department) => {
  console.log(baseAmount, tax, department)
  return baseAmount + tax
})

console.log(salary)

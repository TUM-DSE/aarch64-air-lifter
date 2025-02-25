### Directory Structure

`common`: Contains code used for testing shared between test modules.:q

`completeness`: Contains code used to test if lifter can handle binaries. To execute completeness tests, create a `bin` directory in the test directory and place any elf you want to test the lifter. Afterwards, run the completeness module.
`label resolver`: Contains test code for the lifter's label resolver.
`lifter`: contains e2e-tests for the entire lifter.

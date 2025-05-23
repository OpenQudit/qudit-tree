use qudit_core::matrix::{MatMut, MatRef};
use qudit_core::matrix::{SymSqMatMatMut, SymSqMatMatRef};
use qudit_core::matrix::{MatVecMut, MatVecRef};
use qudit_core::accel::fused_reshape_permute_reshape_into_prepare;
use qudit_core::accel::fused_reshape_permute_reshape_into_impl;
use qudit_core::ComplexScalar;
use crate::bytecode::SizedMatrixBuffer;
use qudit_core::memory::MemoryBuffer;

pub struct FRPRStruct {
    pub len: usize,
    pub ins: [isize; 64],
    pub outs: [isize; 64],
    pub dims: [usize; 64],
    pub input: SizedMatrixBuffer,
    pub out: SizedMatrixBuffer,
}

impl FRPRStruct {
    pub fn new(
        input: SizedMatrixBuffer,
        shape: &Vec<usize>,
        perm: &Vec<usize>,
        out: SizedMatrixBuffer,
    ) -> Self {
        // TODO: Extract 64 to a library level constact (remove magic number)
        let (ins, outs, dims) = fused_reshape_permute_reshape_into_prepare(
            input.nrows,
            input.ncols,
            input.col_stride,
            out.nrows,
            out.ncols,
            out.col_stride,
            shape,
            perm,
        );
        let len = ins.len();
        if len > 64 {
            // TODO: Better error message
            panic!("Too many indices in FRPR operaiton!");
        }
        let mut array_ins = [0; 64];
        for (i, v) in ins.iter().enumerate() {
            array_ins[i] = *v;
        }
        let mut array_outs = [0; 64];
        for (i, v) in outs.iter().enumerate() {
            array_outs[i] = *v;
        }
        let mut array_dims = [0; 64];
        for (i, v) in dims.iter().enumerate() {
            array_dims[i] = *v;
        }
        Self {
            len,
            ins: array_ins,
            outs: array_outs,
            dims: array_dims,
            input,
            out,
        }
    }

    #[inline(always)]
    fn calculate_unitary<C: ComplexScalar>(
        &self,
        input: MatRef<C>,
        out: MatMut<C>,
    ) {
        // Safety: Ins, outs, dims were generated by fused_reshape_permuted_reshape_into_prepare
        // for the same sized input and output matrices with same strides.
        unsafe {
            fused_reshape_permute_reshape_into_impl(
                input,
                out,
                &self.ins[..self.len],
                &self.outs[..self.len],
                &self.dims[..self.len],
            );
        }
    }

    #[inline(always)]
    fn calculate_gradient<C: ComplexScalar>(
        &self,
        input: MatVecRef<C>,
        mut out: MatVecMut<C>,
    ) {
        // TODO: Potential optimization, num_params can be another stride to be
        // optimized
        for i in 0..self.input.num_params {
            let input_gradref = input.mat_ref(i);
            let out_gradmut = out.mat_mut(i);

            // Safety: Ins, outs, dims were generated by fused_reshape_permuted_reshape_into_prepare
            // for the same sized input and output matrices with same strides.
            unsafe {
                fused_reshape_permute_reshape_into_impl(
                    input_gradref,
                    out_gradmut,
                    &self.ins[..self.len],
                    &self.outs[..self.len],
                    &self.dims[..self.len],
                );
            }
        }
    }

    #[inline(always)]
    fn calculate_hessian<C: ComplexScalar>(
        &self,
        input: SymSqMatMatRef<C>,
        out: SymSqMatMatMut<C>,
    ) {
        for p1 in 0..self.input.num_params {
            for p2 in p1..self.input.num_params {
                let input_hessref = input.mat_ref(p1, p2);
                let out_hessmut = out.mat_mut(p1, p2);

                // Safety: Ins, outs, dims were generated by fused_reshape_permuted_reshape_into_prepare
                // for the same sized input and output matrices with same strides.
                unsafe {
                    fused_reshape_permute_reshape_into_impl(
                        input_hessref,
                        out_hessmut,
                        &self.ins[..self.len],
                        &self.outs[..self.len],
                        &self.dims[..self.len],
                    );
                }
            }
        }
    }

    #[inline(always)]
    pub fn execute_unitary<C: ComplexScalar>(&self, memory: &mut MemoryBuffer<C>) {
        let input_matref = self.input.as_matref::<C>(memory);
        let out_matmut = self.out.as_matmut::<C>(memory);
        self.calculate_unitary(input_matref, out_matmut);
    }

    #[inline(always)]
    pub fn execute_unitary_and_gradient<C: ComplexScalar>(
        &self,
        memory: &mut MemoryBuffer<C>,
    ) {
        let input_matref = self.input.as_matref::<C>(memory);
        let input_gradref = self.input.as_matvecref::<C>(memory);
        let out_matmut = self.out.as_matmut::<C>(memory);
        let out_gradmut = self.out.as_matvecmut::<C>(memory);
        self.calculate_unitary(input_matref, out_matmut);
        self.calculate_gradient(input_gradref, out_gradmut);
    }

    #[inline(always)]
    pub fn execute_unitary_gradient_and_hessian<C: ComplexScalar>(
        &self,
        memory: &mut MemoryBuffer<C>,
    ) {
        let input_matref = self.input.as_matref::<C>(memory);
        let input_gradref = self.input.as_matvecref::<C>(memory);
        let input_hessref = self.input.as_symsqmatref::<C>(memory);
        let out_matmut = self.out.as_matmut::<C>(memory);
        let out_gradmut = self.out.as_matvecmut::<C>(memory);
        let out_hessmut = self.out.as_symsqmatmut::<C>(memory);
        self.calculate_unitary(input_matref, out_matmut);
        self.calculate_gradient(input_gradref, out_gradmut);
        self.calculate_hessian(input_hessref, out_hessmut);
    }

    #[inline(always)]
    pub fn execute_unitary_into<C: ComplexScalar>(
        &self,
        memory: &mut MemoryBuffer<C>,
        out: MatMut<C>,
    ) {
        let input_matref = self.input.as_matref::<C>(memory);
        self.calculate_unitary(input_matref, out);
    }

    #[inline(always)]
    pub fn execute_unitary_and_gradient_into<C: ComplexScalar>(
        &self,
        memory: &mut MemoryBuffer<C>,
        out: MatMut<C>,
        out_grad: MatVecMut<C>,
    ) {
        let input_matref = self.input.as_matref::<C>(memory);
        let input_gradref = self.input.as_matvecref::<C>(memory);
        self.calculate_unitary(input_matref, out);
        self.calculate_gradient(input_gradref, out_grad);
    }

    #[inline(always)]
    pub fn execute_unitary_gradient_and_hessian_into<C: ComplexScalar>(
        &self,
        memory: &mut MemoryBuffer<C>,
        out: MatMut<C>,
        out_grad: MatVecMut<C>,
        out_hess: SymSqMatMatMut<C>,
    ) {
        let input_matref = self.input.as_matref::<C>(memory);
        let input_gradref = self.input.as_matvecref::<C>(memory);
        let input_hessref = self.input.as_symsqmatref::<C>(memory);
        self.calculate_unitary(input_matref, out);
        self.calculate_gradient(input_gradref, out_grad);
        self.calculate_hessian(input_hessref, out_hess);
    }
}

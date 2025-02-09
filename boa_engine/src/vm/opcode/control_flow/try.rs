use crate::{
    vm::{
        call_frame::{EnvStackEntry, FinallyAddresses},
        opcode::Operation,
        FinallyReturn, ShouldExit,
    },
    Context, JsResult,
};

/// `TryStart` implements the Opcode Operation for `Opcode::TryStart`
///
/// Operation:
///  - Start of a try block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TryStart;

impl Operation for TryStart {
    const NAME: &'static str = "TryStart";
    const INSTRUCTION: &'static str = "INST - TryStart";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let catch = context.vm.read::<u32>();
        let finally = context.vm.read::<u32>();

        // If a finally exists, push the env to the stack before the try.
        if finally != u32::MAX {
            context.vm.frame_mut().env_stack.push(
                EnvStackEntry::default()
                    .with_finally_flag()
                    .with_start_address(finally),
            );
        }

        context.vm.frame_mut().finally_return = FinallyReturn::None;
        context
            .vm
            .frame_mut()
            .env_stack
            .push(EnvStackEntry::new(catch, finally).with_try_flag());

        let finally = if finally == u32::MAX {
            None
        } else {
            Some(finally)
        };
        context
            .vm
            .frame_mut()
            .try_catch
            .push(FinallyAddresses::new(finally));
        Ok(ShouldExit::False)
    }
}

/// `TryEnd` implements the Opcode Operation for `Opcode::TryEnd`
///
/// Operation:
///  - End of a try block
#[derive(Debug, Clone, Copy)]
pub(crate) struct TryEnd;

impl Operation for TryEnd {
    const NAME: &'static str = "TryEnd";
    const INSTRUCTION: &'static str = "INST - TryEnd";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        context.vm.frame_mut().try_catch.pop();
        let mut envs_to_pop = 0_usize;
        while let Some(env_entry) = context.vm.frame_mut().env_stack.pop() {
            envs_to_pop += env_entry.env_num();

            if env_entry.is_try_env() {
                break;
            }
        }

        let env_truncation_len = context.realm.environments.len().saturating_sub(envs_to_pop);
        context.realm.environments.truncate(env_truncation_len);

        context.vm.frame_mut().finally_return = FinallyReturn::None;
        Ok(ShouldExit::False)
    }
}

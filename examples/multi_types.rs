use std::time::{Duration, Instant};

use sans_io_runtime::{
    backend::MioBackend, Controller, Task, TaskGroup, TaskGroupInput, TaskGroupOutputsState,
    TaskInput, TaskOutput, WorkerInner, WorkerInnerInput, WorkerInnerOutput,
};

type ICfg = ();

#[derive(Clone)]
enum Type1ExtIn {}

#[derive(Clone)]
enum Type2ExtIn {}

#[derive(Clone)]
enum Type1ExtOut {}

#[derive(Clone)]
enum Type2ExtOut {}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum Type1Channel {
    A,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum Type2Channel {
    B,
}

#[derive(Clone)]
enum Type1Event {
    A,
}

#[derive(Clone)]
enum Type2Event {
    B,
}

#[derive(Debug, Clone)]
struct Type1Cfg {}

#[derive(Debug, Clone)]
struct Type2Cfg {}

#[derive(convert_enum::From, convert_enum::TryInto, Clone)]
enum TestExtIn {
    Type1(Type1ExtIn),
    Type2(Type2ExtIn),
}

#[derive(convert_enum::From, convert_enum::TryInto, Clone)]
enum TestExtOut {
    Type1(Type1ExtOut),
    Type2(Type2ExtOut),
}

#[derive(Debug, Hash, PartialEq, Eq, convert_enum::From, convert_enum::TryInto, Clone, Copy)]
enum TestChannel {
    Type1(Type1Channel),
    Type2(Type2Channel),
}

#[derive(Debug, Clone, convert_enum::From, convert_enum::TryInto)]
enum TestSCfg {
    Type1(Type1Cfg),
    Type2(Type2Cfg),
}

#[derive(convert_enum::From, convert_enum::TryInto, Clone)]
enum TestEvent {
    Type1(Type1Event),
    Type2(Type2Event),
}

#[derive(Debug)]
struct Task1 {
    cfg: Type1Cfg,
}

impl Task1 {
    fn new(cfg: Type1Cfg) -> Self {
        Self { cfg }
    }
}

impl Task<Type1Channel, Type1Channel, Type1Event, Type1Event> for Task1 {
    const TYPE: u16 = 0;

    fn on_tick<'a>(
        &mut self,
        _now: Instant,
    ) -> Option<TaskOutput<'a, Type1Channel, Type1Channel, Type1Event>> {
        None
    }

    fn on_event<'b>(
        &mut self,
        _now: Instant,
        _input: TaskInput<'b, Type1Channel, Type1Event>,
    ) -> Option<TaskOutput<'b, Type1Channel, Type1Channel, Type1Event>> {
        None
    }

    fn pop_output<'a>(
        &mut self,
        _now: Instant,
    ) -> Option<TaskOutput<'a, Type1Channel, Type1Channel, Type1Event>> {
        None
    }
}

#[derive(Debug)]
struct Task2 {
    cfg: Type2Cfg,
}

impl Task2 {
    fn new(cfg: Type2Cfg) -> Self {
        Self { cfg }
    }
}

impl Task<Type2Channel, Type2Channel, Type2Event, Type2Event> for Task2 {
    const TYPE: u16 = 1;

    fn on_tick<'a>(
        &mut self,
        _now: Instant,
    ) -> Option<TaskOutput<'a, Type2Channel, Type2Channel, Type2Event>> {
        None
    }

    fn on_event<'b>(
        &mut self,
        _now: Instant,
        _input: TaskInput<'b, Type2Channel, Type2Event>,
    ) -> Option<TaskOutput<'b, Type2Channel, Type2Channel, Type2Event>> {
        None
    }

    fn pop_output<'a>(
        &mut self,
        _now: Instant,
    ) -> Option<TaskOutput<'a, Type2Channel, Type2Channel, Type2Event>> {
        None
    }
}

struct EchoWorkerInner {
    worker: u16,
    echo_type1: TaskGroup<Type1Channel, Type1Channel, Type1Event, Type1Event, Task1, 16>,
    echo_type2: TaskGroup<Type2Channel, Type2Channel, Type2Event, Type2Event, Task2, 16>,
    group_state: TaskGroupOutputsState<2>,
    last_input_index: Option<u16>,
}

impl WorkerInner<TestExtIn, TestExtOut, TestChannel, TestEvent, ICfg, TestSCfg>
    for EchoWorkerInner
{
    fn tasks(&self) -> usize {
        self.echo_type1.tasks() + self.echo_type2.tasks()
    }

    fn worker_index(&self) -> u16 {
        self.worker
    }

    fn build(worker: u16, _cfg: ICfg) -> Self {
        Self {
            worker,
            echo_type1: TaskGroup::new(worker),
            echo_type2: TaskGroup::new(worker),
            last_input_index: None,
            group_state: TaskGroupOutputsState::default(),
        }
    }

    fn spawn(&mut self, _now: Instant, cfg: TestSCfg) {
        match cfg {
            TestSCfg::Type1(cfg) => {
                self.echo_type1.add_task(Task1::new(cfg));
            }
            TestSCfg::Type2(cfg) => {
                self.echo_type2.add_task(Task2::new(cfg));
            }
        }
    }

    fn on_tick<'a>(
        &mut self,
        now: Instant,
    ) -> Option<WorkerInnerOutput<'a, TestExtOut, TestChannel, TestEvent, TestSCfg>> {
        loop {
            match self.group_state.current()? {
                0 => {
                    if let Some(e) = self.group_state.process(self.echo_type1.on_tick(now)) {
                        return Some(e.into());
                    }
                }
                1 => {
                    if let Some(e) = self.group_state.process(self.echo_type2.on_tick(now)) {
                        return Some(e.into());
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    fn on_event<'a>(
        &mut self,
        now: Instant,
        event: WorkerInnerInput<'a, TestExtIn, TestChannel, TestEvent>,
    ) -> Option<WorkerInnerOutput<'a, TestExtOut, TestChannel, TestEvent, TestSCfg>> {
        match event {
            WorkerInnerInput::Task(owner, event) => match owner.group_id() {
                Some(0) => {
                    let res = self
                        .echo_type1
                        .on_event(now, TaskGroupInput(owner, event.convert_into()?))?;
                    self.last_input_index = Some(1);
                    Some(res.into())
                }
                Some(1) => {
                    let res = self
                        .echo_type2
                        .on_event(now, TaskGroupInput(owner, event.convert_into()?))?;
                    self.last_input_index = Some(2);
                    Some(res.into())
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn pop_output<'a>(
        &mut self,
        now: Instant,
    ) -> Option<WorkerInnerOutput<'a, TestExtOut, TestChannel, TestEvent, TestSCfg>> {
        match self.last_input_index? {
            0 => self.echo_type1.pop_output(now).map(|a| a.into()),
            1 => self.echo_type2.pop_output(now).map(|a| a.into()),
            _ => unreachable!(),
        }
    }
}

fn main() {
    env_logger::init();
    println!("{}", std::mem::size_of::<EchoWorkerInner>());
    let mut controller =
        Controller::<TestExtIn, TestExtOut, TestSCfg, TestChannel, TestEvent, 1024>::default();
    controller.add_worker::<_, EchoWorkerInner, MioBackend<1024, 1024>>((), None);
    controller.add_worker::<_, EchoWorkerInner, MioBackend<1024, 1024>>((), None);

    for _i in 0..100 {
        controller.spawn(TestSCfg::Type1(Type1Cfg {}));
        controller.spawn(TestSCfg::Type2(Type2Cfg {}));
    }
    loop {
        controller.process();
        std::thread::sleep(Duration::from_millis(100));
    }
}

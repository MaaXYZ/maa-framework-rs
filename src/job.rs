//! Asynchronous job management for tracking operation status and results.

use std::ops::Deref;

use crate::{
    common::{MaaId, MaaStatus},
    sys, MaaResult,
};

pub type StatusFn = Box<dyn Fn(MaaId) -> MaaStatus + Send + Sync>;
pub type WaitFn = Box<dyn Fn(MaaId) -> MaaStatus + Send + Sync>;

/// An asynchronous operation handle.
///
/// Use this to track the status of controller, resource, and tasker operations.
pub struct Job {
    pub id: MaaId,
    status_fn: StatusFn,
    wait_fn: WaitFn,
}

impl Job {
    /// Create a new Job with custom status/wait functions.
    pub fn new(id: MaaId, status_fn: StatusFn, wait_fn: WaitFn) -> Self {
        Self {
            id,
            status_fn,
            wait_fn,
        }
    }

    /// Create a Job for a Tasker operation.
    pub fn for_tasker(tasker: &crate::tasker::Tasker, id: MaaId) -> Self {
        let tasker1 = tasker.clone();
        let tasker2 = tasker.clone();
        Self {
            id,
            status_fn: Box::new(move |job_id| {
                MaaStatus(unsafe { sys::MaaTaskerStatus(tasker1.raw(), job_id) })
            }),
            wait_fn: Box::new(move |job_id| {
                MaaStatus(unsafe { sys::MaaTaskerWait(tasker2.raw(), job_id) })
            }),
        }
    }

    /// Create a Job for a Controller operation.
    pub fn for_controller(controller: &crate::controller::Controller, id: MaaId) -> Self {
        let controller1 = controller.clone();
        let controller2 = controller.clone();
        Self {
            id,
            status_fn: Box::new(move |job_id| {
                MaaStatus(unsafe { sys::MaaControllerStatus(controller1.raw(), job_id) })
            }),
            wait_fn: Box::new(move |job_id| {
                MaaStatus(unsafe { sys::MaaControllerWait(controller2.raw(), job_id) })
            }),
        }
    }

    /// Create a Job for a Resource operation.
    pub fn for_resource(resource: &crate::resource::Resource, id: MaaId) -> Self {
        let resource1 = resource.clone();
        let resource2 = resource.clone();
        Self {
            id,
            status_fn: Box::new(move |job_id| {
                MaaStatus(unsafe { sys::MaaResourceStatus(resource1.raw(), job_id) })
            }),
            wait_fn: Box::new(move |job_id| {
                MaaStatus(unsafe { sys::MaaResourceWait(resource2.raw(), job_id) })
            }),
        }
    }

    /// Block until the operation completes.
    pub fn wait(&self) -> MaaStatus {
        (self.wait_fn)(self.id)
    }

    /// Get the current status without blocking.
    pub fn status(&self) -> MaaStatus {
        (self.status_fn)(self.id)
    }

    /// Returns `true` if the operation succeeded.
    pub fn succeeded(&self) -> bool {
        self.status() == MaaStatus::SUCCEEDED
    }

    /// Returns `true` if the operation failed.
    pub fn failed(&self) -> bool {
        self.status() == MaaStatus::FAILED
    }

    /// Returns `true` if the operation is running.
    pub fn running(&self) -> bool {
        self.status() == MaaStatus::RUNNING
    }

    /// Returns `true` if the operation is pending.
    pub fn pending(&self) -> bool {
        self.status() == MaaStatus::PENDING
    }

    /// Returns `true` if the operation has completed (success or failure).
    pub fn done(&self) -> bool {
        let s = self.status();
        s == MaaStatus::SUCCEEDED || s == MaaStatus::FAILED
    }
}

/// An asynchronous operation handle with typed result retrieval.
///
/// Similar to [`Job`] but includes a `get()` method to retrieve the operation result.
pub struct JobWithResult<T> {
    job: Job,
    get_fn: Box<dyn Fn(MaaId) -> MaaResult<Option<T>> + Send + Sync>,
}

impl<T> Deref for JobWithResult<T> {
    type Target = Job;

    fn deref(&self) -> &Self::Target {
        &self.job
    }
}

impl<T> JobWithResult<T> {
    /// Create a new JobWithResult with custom status/wait/get functions.
    pub fn new(
        id: MaaId,
        status_fn: StatusFn,
        wait_fn: WaitFn,
        get_fn: impl Fn(MaaId) -> MaaResult<Option<T>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            job: Job::new(id, status_fn, wait_fn),
            get_fn: Box::new(get_fn),
        }
    }

    /// Get the operation result.
    ///
    /// # Arguments
    /// * `wait` - If `true`, blocks until the operation completes before getting the result
    pub fn get(&self, wait: bool) -> MaaResult<Option<T>> {
        if wait {
            self.wait();
        }
        (self.get_fn)(self.job.id)
    }
}

pub type OverridePipelineFn = Box<dyn Fn(MaaId, &str) -> MaaResult<bool> + Send + Sync>;

/// Task job handle with extended capabilities.
///
/// Inherits from [`JobWithResult`], additionally providing task-specific operations.
pub struct TaskJob<T> {
    job: JobWithResult<T>,
    override_fn: OverridePipelineFn,
}

impl<T> Deref for TaskJob<T> {
    type Target = JobWithResult<T>;

    fn deref(&self) -> &Self::Target {
        &self.job
    }
}

impl<T> TaskJob<T> {
    /// Create a new TaskJob.
    pub fn new(job: JobWithResult<T>, override_fn: OverridePipelineFn) -> Self {
        Self { job, override_fn }
    }

    /// Override the pipeline for this task.
    ///
    /// Dynamically modifies the pipeline configuration during task execution.
    ///
    /// # Arguments
    /// * `pipeline_override` - The JSON string for overriding.
    ///
    /// # Returns
    /// * `true` if successful.
    pub fn override_pipeline(&self, pipeline_override: &str) -> MaaResult<bool> {
        (self.override_fn)(self.job.id, pipeline_override)
    }
}

// === Type Aliases ===

/// Controller operation job.
///
/// Returned by controller methods like `post_click()`, `post_swipe()`.
pub type CtrlJob = Job;

/// Resource loading job.
///
/// Returned by resource methods like `post_bundle()`.
pub type ResJob = Job;

/// Task job with result retrieval.
///
/// Returned by `Tasker::post_task()`.
pub type TaskJobWithResult = JobWithResult<crate::common::TaskDetail>;

/// Recognition job with result retrieval.
///
/// Returned by `Tasker::post_recognition()`.
pub type RecoJobWithResult = JobWithResult<crate::common::RecognitionDetail>;

/// Action job with result retrieval.
///
/// Returned by `Tasker::post_action()`.
pub type ActionJobWithResult = JobWithResult<crate::common::ActionDetail>;

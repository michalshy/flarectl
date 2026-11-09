import type { components, paths } from "./schema"

// Short names for the generated API types, so pages don't repeat components["schemas"][...].
type Schemas = components["schemas"]

export type Job = Schemas["Job"]
export type Run = Schemas["Run"]
export type RunDetail = Schemas["RunDetail"]
export type RunStatus = Schemas["RunStatus"]
export type RunFailure = Schemas["RunFailure"]
export type RunTrigger = Schemas["RunTrigger"]
export type Step = Schemas["Step"]
export type StepStatus = Schemas["StepStatus"]
export type Event = Schemas["Event"]

/** Query parameters of GET /v1/runs. */
export type RunsFilter = NonNullable<paths["/v1/runs"]["get"]["parameters"]["query"]>

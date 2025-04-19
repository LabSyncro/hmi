// This file is auto-generated. Do not edit manually.

export enum UserRole {
  SYSADMIN = 'sysadmin',
  LAB_ADMIN = 'lab_admin',
  TEACHER = 'teacher',
  STUDENT = 'student',
}

export enum MaintenanceStatus {
  PENDING = 'pending',
  MAINTAINING = 'maintaining',
  COMPLETED = 'completed',
  CANCELLED = 'cancelled',
}

export enum ActivityType {
  BORROW = 'borrow',
  RETURN = 'return',
  MAINTENANCE = 'maintenance',
  SHIPMENT = 'shipment',
  ASSESSMENT = 'assessment',
}

export enum RequestStatus {
  PENDING = 'pending',
  APPROVED = 'approved',
  REJECTED = 'rejected',
  CANCELLED = 'cancelled',
}

export enum ReservationStatus {
  PENDING = 'pending',
  APPROVED = 'approved',
  READY = 'ready',
  CANCELLED = 'cancelled',
}

export enum AssessmentStatus {
  PENDING = 'pending',
  ASSESSING = 'assessing',
  COMPLETED = 'completed',
  CANCELLED = 'cancelled',
}

export enum DeviceStatus {
  HEALTHY = 'healthy',
  BROKEN = 'broken',
  DISCARDED = 'discarded',
  ASSESSING = 'assessing',
  MAINTAINING = 'maintaining',
  SHIPPING = 'shipping',
  BORROWING = 'borrowing',
  LOST = 'lost',
}

export enum ShipmentStatus {
  PENDING = 'pending',
  SHIPPING = 'shipping',
  COMPLETED = 'completed',
  CANCELLED = 'cancelled',
}

export enum DeviceQuality {
  HEALTHY = 'healthy',
  NEEDS_FIXING = 'needs_fixing',
  BROKEN = 'broken',
  LOST = 'lost',
}

export interface Resources {
  id: number
  createdAt: Date
  name: string
  type: string | null
}

export interface PgStatStatements {
  userid: string | null
  dbid: string | null
  toplevel: boolean | null
  queryid: number | null
  query: string | null
  plans: number | null
  totalPlanTime: number | null
  minPlanTime: number | null
  maxPlanTime: number | null
  meanPlanTime: number | null
  stddevPlanTime: number | null
  calls: number | null
  totalExecTime: number | null
  minExecTime: number | null
  maxExecTime: number | null
  meanExecTime: number | null
  stddevExecTime: number | null
  rows: number | null
  sharedBlksHit: number | null
  sharedBlksRead: number | null
  sharedBlksDirtied: number | null
  sharedBlksWritten: number | null
  localBlksHit: number | null
  localBlksRead: number | null
  localBlksDirtied: number | null
  localBlksWritten: number | null
  tempBlksRead: number | null
  tempBlksWritten: number | null
  blkReadTime: number | null
  blkWriteTime: number | null
  tempBlkReadTime: number | null
  tempBlkWriteTime: number | null
  walRecords: number | null
  walFpi: number | null
  walBytes: number | null
  jitFunctions: number | null
  jitGenerationTime: number | null
  jitInliningCount: number | null
  jitInliningTime: number | null
  jitOptimizationCount: number | null
  jitOptimizationTime: number | null
  jitEmissionCount: number | null
  jitEmissionTime: number | null
}

export interface MfaChallenges {
  id: string
  factorId: string
  createdAt: Date
  verifiedAt: Date | null
  ipAddress: string
  otpCode: string | null
}

export interface Actions {
  id: number
  createdAt: Date
  name: string
}

export interface FlowState {
  id: string
  userId: string | null
  authCode: string
  codeChallengeMethod: string
  codeChallenge: string
  providerType: string
  providerAccessToken: string | null
  providerRefreshToken: string | null
  createdAt: Date | null
  updatedAt: Date | null
  authenticationMethod: string
  authCodeIssuedAt: Date | null
}

export interface Shipments {
  id: string
  senderId: string | null
  receiverId: string | null
  status: ShipmentStatus
  startAt: Date | null
  arriveAt: Date | null
  startLabId: string
  arriveLabId: string
  deviceIds: string[]
}

export interface SsoDomains {
  id: string
  ssoProviderId: string
  domain: string
  createdAt: Date | null
  updatedAt: Date | null
}

export interface HmiCodes {
  code: number
  createdAt: Date
  userId: string | null
  authToken: string | null
  status: string
  updatedAt: Date | null
  expiresAt: Date | null
  labId: string | null
}

export interface Secrets {
  id: string
  name: string | null
  description: string
  secret: string
  keyId: string | null
  nonce: unknown | null
  createdAt: Date
  updatedAt: Date
}

export interface Permissions {
  createdAt: Date
  roleId: number
  resourceId: number
  actionId: number
  priority: number
}

export interface DecryptedKey {
  id: string | null
  status: string | null
  created: Date | null
  expires: Date | null
  keyType: string | null
  keyId: number | null
  keyContext: unknown | null
  name: string | null
  associatedData: string | null
  rawKey: unknown | null
  decryptedRawKey: unknown | null
  rawKeyNonce: unknown | null
  parentKey: string | null
  comment: string | null
}

export interface Extensions {
  id: string
  type: string | null
  settings: any | null
  tenantExternalId: string | null
  insertedAt: Date
  updatedAt: Date
}

export interface Activities {
  id: string
  type: ActivityType
  createdAt: Date
}

export interface MfaFactors {
  id: string
  userId: string
  friendlyName: string | null
  factorType: string
  status: string
  createdAt: Date
  updatedAt: Date
  secret: string | null
  phone: string | null
  lastChallengedAt: Date | null
}

export interface Roles {
  id: number
  createdAt: Date
  key: string
  name: string | null
}

export interface Messages {
  id: number
  topic: string
  extension: string
  insertedAt: Date
  updatedAt: Date
}

export interface HttpRequestQueue {
  id: number
  method: string
  url: string
  headers: any
  body: unknown | null
  timeoutMilliseconds: number
}

export interface Devices {
  createdAt: Date
  kind: string
  labId: string | null
  deletedAt: Date | null
  status: DeviceStatus | null
  id: string
  fullId: string
  printedAt: Date | null
}

export interface Migrations {
  id: number
  name: string
  hash: string
  executedAt: Date | null
}

export interface Instances {
  id: string
  uuid: string | null
  rawBaseConfig: string | null
  createdAt: Date | null
  updatedAt: Date | null
}

export interface RefreshTokens {
  instanceId: string | null
  id: number
  token: string | null
  userId: string | null
  revoked: boolean | null
  createdAt: Date | null
  updatedAt: Date | null
  parent: string | null
  sessionId: string | null
}

export interface UserRoles {
  createdAt: Date
  userId: string
  roleId: number
}

export interface Subscription {
  id: number
  subscriptionId: string
  entity: string
  filters: string[]
  claims: any
  claimsRole: string
  createdAt: Date
}

export interface MfaAmrClaims {
  sessionId: string
  createdAt: Date
  updatedAt: Date
  authenticationMethod: string
  id: string
}

export interface Objects {
  id: string
  bucketId: string | null
  name: string | null
  owner: string | null
  createdAt: Date | null
  updatedAt: Date | null
  lastAccessedAt: Date | null
  metadata: any | null
  pathTokens: string[] | null
  version: string | null
  ownerId: string | null
  userMetadata: any | null
}

export interface RoleHistories {
  createdAt: Date
  granteeId: string
  granterId: string
  permissions: any | null
  effectiveStart: Date
  effectiveEnd: Date
}

export interface DecryptedSecrets {
  id: string | null
  name: string | null
  description: string | null
  secret: string | null
  decryptedSecret: string | null
  keyId: string | null
  nonce: unknown | null
  createdAt: Date | null
  updatedAt: Date | null
}

export interface SchemaMigrations {
  version: number
  insertedAt: Date | null
}

export interface Sessions {
  id: string
  userId: string
  createdAt: Date | null
  updatedAt: Date | null
  factorId: string | null
  aal: string | null
  notAfter: Date | null
  refreshedAt: Date | null
  userAgent: string | null
  ip: string | null
  tag: string | null
}

export interface S3MultipartUploadsParts {
  id: string
  uploadId: string
  size: number
  partNumber: number
  bucketId: string
  key: string
  etag: string
  ownerId: string | null
  version: string
  createdAt: Date
}

export interface Tenants {
  id: string
  name: string | null
  externalId: string | null
  jwtSecret: string | null
  maxConcurrentUsers: number
  insertedAt: Date
  updatedAt: Date
  maxEventsPerSecond: number
  postgresCdcDefault: string | null
  maxBytesPerSecond: number
  maxChannelsPerClient: number
  maxJoinsPerSecond: number
  suspend: boolean | null
  jwtJwks: any | null
  notifyPrivateAlpha: boolean | null
}

export interface HttpResponse {
  id: number | null
  statusCode: number | null
  contentType: string | null
  headers: any | null
  content: string | null
  timedOut: boolean | null
  errorMsg: string | null
  created: Date
}

export interface Key {
  id: string
  status: string | null
  created: Date
  expires: Date | null
  keyType: string | null
  keyId: number | null
  keyContext: unknown | null
  name: string | null
  associatedData: string | null
  rawKey: unknown | null
  rawKeyNonce: unknown | null
  parentKey: string | null
  comment: string | null
  userData: string | null
}

export interface ValidKey {
  id: string | null
  name: string | null
  status: string | null
  keyType: string | null
  keyId: number | null
  keyContext: unknown | null
  created: Date | null
  expires: Date | null
  associatedData: string | null
}

export interface Receipts {
  id: string
  borrowerId: string
  borrowCheckerId: string
  borrowedLabId: string
}

export interface Users {
  instanceId: string | null
  id: string
  aud: string | null
  role: string | null
  email: string | null
  encryptedPassword: string | null
  emailConfirmedAt: Date | null
  invitedAt: Date | null
  confirmationToken: string | null
  confirmationSentAt: Date | null
  recoveryToken: string | null
  recoverySentAt: Date | null
  emailChangeTokenNew: string | null
  emailChange: string | null
  emailChangeSentAt: Date | null
  lastSignInAt: Date | null
  rawAppMetaData: any | null
  rawUserMetaData: any | null
  isSuperAdmin: boolean | null
  createdAt: Date | null
  updatedAt: Date | null
  phone: string | null
  phoneConfirmedAt: Date | null
  phoneChange: string | null
  phoneChangeToken: string | null
  phoneChangeSentAt: Date | null
  confirmedAt: Date | null
  emailChangeTokenCurrent: string | null
  emailChangeConfirmStatus: number | null
  bannedUntil: Date | null
  reauthenticationToken: string | null
  reauthenticationSentAt: Date | null
  isSsoUser: boolean
  deletedAt: Date | null
  isAnonymous: boolean
}

export interface MaskColumns {
  attname: string | null
  attrelid: string | null
  keyId: string | null
  keyIdColumn: string | null
  associatedColumns: string | null
  nonceColumn: string | null
  formatType: string | null
}

export interface SamlProviders {
  id: string
  ssoProviderId: string
  entityId: string
  metadataXml: string
  metadataUrl: string | null
  attributeMapping: any | null
  createdAt: Date | null
  updatedAt: Date | null
  nameIdFormat: string | null
}

export interface PgStatStatementsInfo {
  dealloc: number | null
  statsReset: Date | null
}

export interface InventoryAssessments {
  id: string
  finishedAt: Date | null
  labId: string
  accountantId: string | null
  status: AssessmentStatus
  deviceIds: string[]
}

export interface Hooks {
  id: number
  hookTableId: number
  hookName: string
  createdAt: Date
  requestId: number | null
}

export interface SchemaMigrations_Auth {
  version: string
}

export interface SchemaMigrations_Realtime {
  version: number
  insertedAt: Date | null
}

export interface OneTimeTokens {
  id: string
  userId: string
  tokenType: string
  tokenHash: string
  relatesTo: string
  createdAt: Date
  updatedAt: Date
}

export interface ReceiptsDevices {
  createdAt: Date
  receiptId: string | null
  deviceId: string
  prevQuality: DeviceQuality | null
  borrowId: string | null
  returnId: string | null
  expectedReturnedAt: Date
  id: string
  expectedReturnedLabId: string | null
  returnCheckerId: string | null
  note: string | null
  afterQuality: DeviceQuality | null
}

export interface Users_Public {
  id: string
  createdAt: Date
  name: string
  meta: any
  email: string | null
  password: string | null
  image: string | null
  tel: string | null
  deletedAt: Date | null
  lastActiveAt: Date | null
}

export interface UsedQrTokens {
  id: number
  createdAt: Date
  token: string
  userId: string
}

export interface DeviceKinds {
  createdAt: Date
  categoryId: number | null
  name: string | null
  meta: any
  image: any | null
  brand: string | null
  manufacturer: string | null
  description: string | null
  datasheet: string | null
  unit: string | null
  price: string | null
  deletedAt: Date | null
  id: string
  allowedBorrowRoles: string[] | null
  allowedViewRoles: string[] | null
  isBorrowableLabOnly: boolean
}

export interface Migrations_SupabaseFunctions {
  version: string
  insertedAt: Date
}

export interface AuditLogEntries {
  instanceId: string | null
  id: string
  payload: any | null
  createdAt: Date | null
  ipAddress: string
}

export interface S3MultipartUploads {
  id: string
  inProgressSize: number
  uploadSignature: string
  bucketId: string
  key: string
  version: string
  ownerId: string | null
  createdAt: Date
  userMetadata: any | null
}

export interface SamlRelayStates {
  id: string
  ssoProviderId: string
  requestId: string
  forEmail: string | null
  redirectTo: string | null
  createdAt: Date | null
  updatedAt: Date | null
  flowStateId: string | null
}

export interface MaskingRule {
  attrelid: string | null
  attnum: number | null
  relnamespace: string | null
  relname: string | null
  attname: string | null
  formatType: string | null
  colDescription: string | null
  keyIdColumn: string | null
  keyId: string | null
  associatedColumns: string | null
  nonceColumn: string | null
  viewName: string | null
  priority: number | null
  securityInvoker: boolean | null
}

export interface Identities {
  providerId: string
  userId: string
  identityData: any
  provider: string
  lastSignInAt: Date | null
  createdAt: Date | null
  updatedAt: Date | null
  email: string | null
  id: string
}

export interface Buckets {
  id: string
  name: string
  owner: string | null
  createdAt: Date | null
  updatedAt: Date | null
  public: boolean | null
  avifAutodetection: boolean | null
  fileSizeLimit: number | null
  allowedMimeTypes: string[] | null
  ownerId: string | null
}

export interface Maintenances {
  id: string
  status: MaintenanceStatus
  maintainerId: string | null
  startAt: Date | null
  completeAt: Date | null
  labId: string
  deviceIds: string[]
}

export interface Labs {
  id: string
  createdAt: Date
  name: string | null
  faculty: string | null
  room: string | null
  branch: string | null
  timetable: any
  adminId: string | null
  deletedAt: Date | null
}

export interface SsoProviders {
  id: string
  resourceId: string | null
  createdAt: Date | null
  updatedAt: Date | null
}

export interface Categories {
  id: number
  createdAt: Date
  name: string
  quantity: number | null
}


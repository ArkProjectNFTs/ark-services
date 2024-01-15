import { Period, RestApi, UsagePlan } from "aws-cdk-lib/aws-apigateway";

export function setupApiPlans(
  api: RestApi,
  environment: string
): { basicPlan: UsagePlan; payAsYouGoPlan: UsagePlan; adminPlan: UsagePlan } {
  const basicPlan = createBasicPlan(api, environment);
  const payAsYouGoPlan = createPayAsYouGoPlan(api, environment);
  const adminPlan = createAdminPlan(api, environment);

  return { basicPlan, payAsYouGoPlan, adminPlan };
}

function createBasicPlan(api: RestApi, environment: string): UsagePlan {
  return api.addUsagePlan(`ark-basic-plan-${environment}`, {
    name: `ark-basic-plan-${environment}`,
    throttle: {
      rateLimit: 5, // 5 requests per second
      burstLimit: 2, // Allow a burst of 2 requests
    },
    quota: {
      limit: 100000, // 100000 requests per month
      period: Period.MONTH,
    },
  });
}

function createPayAsYouGoPlan(api: RestApi, environment: string): UsagePlan {
  return api.addUsagePlan(`ark-pay-as-you-go-plan-${environment}`, {
    name: `ark-pay-as-you-go-plan-${environment}`,
    throttle: {
      rateLimit: 100, // 100 requests per second
      burstLimit: 50, // Allow a burst of 50 requests
    },
  });
}

function createAdminPlan(api: RestApi, environment: string): UsagePlan {
  return api.addUsagePlan(`ark-admin-plan-${environment}`, {
    name: `ark-admin-plan-${environment}`,
  });
}

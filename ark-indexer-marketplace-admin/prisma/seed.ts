// import { db } from "../src/server/db";

// async function createUsers() {
//   const user = await prisma.user.upsert({
//     where: {
//       email: "user@example.com",
//     },
//     update: {},
//     create: {
//       email: "user@example.com",
//       role: "USER"
//     },
//   });

//   console.log(`🤷‍♂️  Created user '${user.email}'`);
// }

// async function main() {
//   console.log();
//   await createUsers();
// }

// main()
//   .catch((e) => {
//     console.error(e);
//     process.exit(1);
//   })
//   .finally(async () => {
//     await prisma.$disconnect();
//   });

import { PrismaAdapter } from "@auth/prisma-adapter";
import { type NextAuthOptions, type User } from "next-auth";
import { getServerSession } from "next-auth/next";
import EmailProvider from "next-auth/providers/email";
import { Resend } from "resend";

import LoginVerificationCode from "~/emails/login-verification-code";
import { env } from "~/env.mjs";
import { db } from "./db";

type UserId = string;

declare module "next-auth/jwt" {
  interface JWT {
    id: UserId;
  }
}

declare module "next-auth" {
  interface Session {
    user: User & {
      id: UserId;
    };
  }
}

const resend = new Resend(env.RESEND_API_KEY);

export const authOptions: NextAuthOptions = {
  adapter: PrismaAdapter(db),
  session: {
    strategy: "jwt",
  },
  providers: [
    EmailProvider({
      async sendVerificationRequest({ identifier, url }) {
        await resend.sendEmail({
          from: "ArkAdmin <notifications@arkproject.dev>",
          to: identifier,
          subject: "Login to ArkAdmin",
          react: LoginVerificationCode({ url }),
        });
      },
    }),
  ],
  pages: {
    signIn: "/login",
    error: "/login/error",
    verifyRequest: "/login/verify",
  },
  callbacks: {
    signIn({ user }) {
      const email = user.email ?? "";
      const [, domain] = email.split("@");

      return domain === "screenshot.co";
    },
    session({ token, session }) {
      if (token && session.user) {
        session.user.id = token.id;
        session.user.name = token.name;
        session.user.email = token.email;
        session.user.image = token.picture;
      }

      return session;
    },
    async jwt({ token, user }) {
      const dbUser = await db.user.findFirst({
        where: {
          email: token.email,
        },
      });

      if (!dbUser) {
        if (user) {
          token.id = user?.id;
        }

        return token;
      }

      return {
        id: dbUser.id,
        name: dbUser.name,
        email: dbUser.email,
        picture: dbUser.image,
      };
    },
  },
};

export const getServerAuthSession = () => getServerSession(authOptions);

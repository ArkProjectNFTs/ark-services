import {
  Body,
  Container,
  Head,
  Heading,
  Html,
  Img,
  Link,
  Preview,
  Text,
} from "@react-email/components";
import { Tailwind } from "@react-email/tailwind";

const baseUrl = process.env.VERCEL_URL
  ? `https://${process.env.VERCEL_URL}`
  : "https://www.arkproject.dev/";

const h1 = {
  fontFamily:
    "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif",
};

const buttonLink = {
  fontFamily:
    "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif",
};

const link = {
  fontFamily:
    "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif",
  textDecoration: "underline",
};

const text = {
  fontFamily:
    "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif",
};

const footer = {
  borderTop: "1px solid #dfe1e4",
  fontFamily:
    "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif",
};


export const LoginVerificationCode = ({ url }: { url: string }) => (
  <Html>
    <Head />
    <Preview>Log in with this magic link</Preview>
    <Tailwind>
      <Body className="bg-white m-6">
        <Container className="mx-auto border-solid border-gray-200 rounded-md" style={{ borderWidth: 1 }}>
          <Container className="px-10 py-8">
            <Img
              src={`${baseUrl}/emails/arkproject-logo.png`}
              width="229"
              height="30"
              alt="ArkProject"
            />
            <Heading className="text-[#282a30] font-semibold text-xl my-8" style={h1}>Your login link to ArkProject</Heading>
            <Link
              href={url}
              target="_blank"
              className="bg-[#f8545c] text-white px-6 py-3 rounded-md font-bold text-sm mb-4 inline-block"
              style={buttonLink}
            >
              Login to ArkProject
            </Link>
            <Text style={text} className="m-0 mb-8 text-[#282a30]">
              If you didn&apos;t try to login, you can safely ignore this email.
            </Text>
            <Text style={footer} className="m-0">
              <Link
                href="https://arkproject.dev"
                target="_blank"
                className="no-underline text-inherit inline-block mt-6 text-[#282a30]"
                style={link}
              >
                ArkProject
              </Link>
            </Text>
          </Container>
        </Container>
      </Body>
    </Tailwind>
  </Html>
);

export default LoginVerificationCode;

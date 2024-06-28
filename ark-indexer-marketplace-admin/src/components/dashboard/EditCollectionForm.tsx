/* eslint-disable @next/next/no-img-element */
/* eslint-disable @typescript-eslint/no-misused-promises */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
"use client";

import { useState } from "react";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";

import { Button } from "~/components/ui/button";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
} from "~/components/ui/form";
import { Input } from "~/components/ui/input";
import { Switch } from "~/components/ui/switch";
import { api } from "~/trpc/react";
import type { Contract } from "~/types";

const editCollectionFormSchema = z.object({
  name: z.string(),
  saveImages: z.boolean(),
  symbol: z.string(),
  isSpam: z.boolean(),
  isNSFW: z.boolean(),
  isVerified: z.boolean(),
  image: z.string().optional(),
});

type EditCollectionFormValues = z.infer<typeof editCollectionFormSchema>;

async function uploadFileToS3(file: File, presignedUrl: string) {
  try {
    const response = await fetch(presignedUrl, {
      method: "PUT",
      headers: {
        "Content-Type": file.type,
      },
      body: file,
    });

    if (!response.ok) {
      throw new Error("Failed to upload file!");
    }
  } catch (error) {
    console.error("Error uploading file:", error);
  }
}

export default function CollectionForm(props: { contract?: Contract }) {
  const [avatarMediaKey, setAvatarMediaKey] = useState<string>();
  const [file, setFile] = useState<File>();

  const form = useForm<EditCollectionFormValues>({
    resolver: zodResolver(editCollectionFormSchema),
    defaultValues: {
      saveImages: props.contract?.save_images ?? false,
      name: props.contract?.contract_name,
      symbol: props.contract?.contract_symbol,
      isSpam: props.contract?.is_spam ?? false,
      isNSFW: props.contract?.is_nsfw ?? false,
      isVerified: props.contract?.is_verified ?? false,
      image: props.contract?.contract_image,
    },
  });

  const { data: avatarPresignedUri } =
    api.media.generateMediaPresignedUri.useQuery(
      {
        mediaKey: avatarMediaKey ?? "",
      },
      {
        enabled: !!avatarMediaKey,
      },
    );

  const { mutateAsync, isLoading } = api.contract.updateContract.useMutation(
    {},
  );

  return (
    <Form {...form}>
      <form
        // onSubmit={async () => {
        //   if (file && avatarPresignedUri) {
        //     try {
        //       await uploadFileToS3(file, avatarPresignedUri);
        //       form.setValue(
        //         "image",
        //         `https://media.arkproject.dev/${avatarMediaKey}`,
        //       );
        //     } catch {}

        //     console.log("File uploaded to S3");
        //   }

        //   return form.handleSubmit(onSubmit);
        // }}
        className="space-y-8"
      >
        {props.contract?.contract_image && (
          <div>
            <img
              className="rounded-md border-2 border-muted p-1"
              src={props.contract?.contract_image}
              alt="Image Preview"
              width={200}
            />
            {/* <Button variant="secondary" className="mt-2 w-[200px]">
              Delete
            </Button> */}
          </div>
        )}
        <FormItem>
          <FormLabel htmlFor="avatar-image">Image</FormLabel>
          <Input
            onChange={(event) => {
              if (event.target?.files && event.target.files.length > 0) {
                const file = event.target.files[0];
                const fileExt = file?.name.split(".").pop();
                setAvatarMediaKey(
                  `contracts/${props.contract?.contract_address}/avatar.${fileExt}`,
                );
                setFile(file);
              }
            }}
            id="avatar-image"
            type="file"
          />
        </FormItem>
        <FormItem>
          <FormLabel>Type</FormLabel>
          <Input
            readOnly={true}
            contentEditable={false}
            value={props.contract?.contract_type}
            disabled={true}
          />
        </FormItem>
        <div className="grid grid-cols-2 gap-4">
          <div className="grid gap-2">
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel htmlFor={field.name}>Name</FormLabel>
                  <Input {...field} />
                </FormItem>
              )}
            />
          </div>
          <div className="grid gap-2">
            <FormField
              control={form.control}
              name="symbol"
              render={({ field }) => (
                <FormItem>
                  <FormLabel htmlFor={field.name}>Symbol</FormLabel>
                  <Input {...field} />
                </FormItem>
              )}
            />
          </div>
        </div>
        <div>
          <h3 className="mb-4 text-lg font-medium">Indexer Settings</h3>
          <div className="space-y-4">
            <FormField
              control={form.control}
              name="saveImages"
              render={({ field }) => (
                <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                  <div className="space-y-0.5">
                    <FormLabel className="text-base">☁️ Save Images</FormLabel>
                    <FormDescription>
                      Save images to AWS S3 to cache them for faster loading
                    </FormDescription>
                  </div>
                  <FormControl>
                    <Switch
                      checked={field.value}
                      onCheckedChange={field.onChange}
                    />
                  </FormControl>
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="isNSFW"
              render={({ field }) => (
                <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                  <div className="space-y-0.5">
                    <FormLabel className="text-base">🔞 Is NSFW</FormLabel>
                    <FormDescription>
                      Mark this collection as NSFW
                    </FormDescription>
                  </div>
                  <FormControl>
                    <Switch
                      checked={field.value}
                      onCheckedChange={field.onChange}
                    />
                  </FormControl>
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="isSpam"
              render={({ field }) => (
                <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                  <div className="space-y-0.5">
                    <FormLabel className="text-base">⛔ Is Spam</FormLabel>
                    <FormDescription>
                      Mark this collection as spam
                    </FormDescription>
                  </div>
                  <FormControl>
                    <Switch
                      checked={field.value}
                      onCheckedChange={field.onChange}
                    />
                  </FormControl>
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="isVerified"
              render={({ field }) => (
                <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                  <div className="space-y-0.5">
                    <FormLabel className="text-base">✅ Is Verified</FormLabel>
                    <FormDescription>
                      Mark this collection as verified
                    </FormDescription>
                  </div>
                  <FormControl>
                    <Switch
                      checked={field.value}
                      onCheckedChange={field.onChange}
                    />
                  </FormControl>
                </FormItem>
              )}
            />
          </div>
        </div>

        <div>
          <Button
            type="button"
            onClick={async () => {
              if (props.contract?.contract_address) {
                if (file && avatarPresignedUri) {
                  await uploadFileToS3(file, avatarPresignedUri);
                }

                const data = form.getValues();
                await mutateAsync({
                  contractAddress: props.contract.contract_address,
                  image: avatarMediaKey
                    ? `https://media.arkproject.dev/${avatarMediaKey}`
                    : undefined,
                  isNSFW: data.isNSFW,
                  isSpam: data.isSpam,
                  isVerified: data.isVerified,
                  name: data.name,
                  saveImages: data.saveImages,
                  symbol: data.symbol,
                });
              }
            }}
          >
            Update Collection
          </Button>
        </div>
      </form>
    </Form>
  );
}
